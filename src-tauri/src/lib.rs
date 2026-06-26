use anyhow::{anyhow, Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    fs,
    path::PathBuf,
    process::Command,
    sync::{Arc, Mutex, MutexGuard},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, PhysicalPosition, Position, State, WebviewWindow, WindowEvent,
};
use tauri_plugin_notification::NotificationExt;

const ALLOWED_INTERVAL_SECONDS: [u64; 6] = [MIN_INTERVAL_SECONDS, 10, 15, 20, 25, 30];
const MIN_INTERVAL_SECONDS: u64 = 5;
const DEFAULT_INTERVAL_SECONDS: u64 = 20;
const CONNECT_TIMEOUT_SECONDS: u64 = 3;
const REQUEST_TIMEOUT_SECONDS: u64 = 5;
const CONNECTIVITY_MONITOR_INTERVAL_SECONDS: u64 = 3;
const MAX_LOGS: usize = 30;
const WINDOW_MARGIN_X_PX: i32 = 16;
const WINDOW_MARGIN_Y_PX: i32 = 36;
const CONNECTIVITY_URL: &str = "https://www.gstatic.com/generate_204";
const KICK_URL: &str = "https://selfcare.hutch.lk/selfcare/login.html";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum ServiceMachineState {
    Stopped,
    Starting,
    Running,
    Stopping,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum WifiStatus {
    Connected,
    Disconnected,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum InternetStatus {
    Online,
    Offline,
    Unknown,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct LogEvent {
    id: u64,
    timestamp_ms: u64,
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppSettings {
    interval_seconds: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ServiceSnapshot {
    current_state: ServiceMachineState,
    wifi_status: WifiStatus,
    internet_status: InternetStatus,
    last_kick_time_ms: Option<u64>,
    interval_seconds: u64,
    logs: Vec<LogEvent>,
    error_message: Option<String>,
}

struct InnerState {
    current_state: ServiceMachineState,
    wifi_status: WifiStatus,
    internet_status: InternetStatus,
    last_kick_time: Option<SystemTime>,
    interval_seconds: u64,
    logs: VecDeque<LogEvent>,
    next_log_id: u64,
    error_message: Option<String>,
    worker_handle: Option<tauri::async_runtime::JoinHandle<()>>,
    client: Client,
}

impl InnerState {
    fn new(client: Client) -> Self {
        let state = Self {
            current_state: ServiceMachineState::Stopped,
            wifi_status: WifiStatus::Unknown,
            internet_status: InternetStatus::Unknown,
            last_kick_time: None,
            interval_seconds: DEFAULT_INTERVAL_SECONDS,
            logs: VecDeque::with_capacity(MAX_LOGS),
            next_log_id: 1,
            error_message: None,
            worker_handle: None,
            client,
        };
        state
    }

    fn snapshot(&self) -> ServiceSnapshot {
        ServiceSnapshot {
            current_state: self.current_state,
            wifi_status: self.wifi_status,
            internet_status: self.internet_status,
            last_kick_time_ms: self.last_kick_time.map(system_time_to_ms),
            interval_seconds: self.interval_seconds,
            logs: self.logs.iter().cloned().collect(),
            error_message: self.error_message.clone(),
        }
    }

    fn push_log(&mut self, message: impl Into<String>) {
        let entry = LogEvent {
            id: self.next_log_id,
            timestamp_ms: now_ms(),
            message: message.into(),
        };
        self.next_log_id += 1;
        self.logs.push_front(entry);
        if self.logs.len() > MAX_LOGS {
            self.logs.truncate(MAX_LOGS);
        }
    }

    fn transition(&mut self, next: ServiceMachineState) -> Result<()> {
        if is_valid_transition(self.current_state, next) {
            self.current_state = next;
            Ok(())
        } else {
            Err(anyhow!(
                "Invalid transition: {:?} -> {:?}",
                self.current_state,
                next
            ))
        }
    }
}

#[derive(Clone)]
struct SharedState(Arc<Mutex<InnerState>>);

impl SharedState {
    fn new(client: Client) -> Self {
        Self(Arc::new(Mutex::new(InnerState::new(client))))
    }

    fn lock(&self) -> MutexGuard<'_, InnerState> {
        self.0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn snapshot(&self) -> ServiceSnapshot {
        self.lock().snapshot()
    }
}

fn is_valid_transition(from: ServiceMachineState, to: ServiceMachineState) -> bool {
    matches!(
        (from, to),
        (ServiceMachineState::Stopped, ServiceMachineState::Starting)
            | (ServiceMachineState::Starting, ServiceMachineState::Running)
            | (ServiceMachineState::Starting, ServiceMachineState::Stopped)
            | (ServiceMachineState::Running, ServiceMachineState::Stopping)
            | (ServiceMachineState::Stopping, ServiceMachineState::Stopped)
            | (_, ServiceMachineState::Error)
            | (ServiceMachineState::Error, ServiceMachineState::Stopped)
    )
}

fn sanitize_interval_seconds(interval_seconds: u64) -> u64 {
    if ALLOWED_INTERVAL_SECONDS.contains(&interval_seconds) {
        interval_seconds
    } else {
        DEFAULT_INTERVAL_SECONDS
    }
}

fn service_owns_connectivity(state: ServiceMachineState) -> bool {
    matches!(
        state,
        ServiceMachineState::Starting
            | ServiceMachineState::Running
            | ServiceMachineState::Stopping
    )
}

fn now_ms() -> u64 {
    system_time_to_ms(SystemTime::now())
}

fn format_latency(elapsed: Duration) -> String {
    format!("{}ms", elapsed.as_millis())
}

fn settings_path(app: &AppHandle) -> Result<PathBuf> {
    Ok(app.path().app_config_dir()?.join("settings.json"))
}

fn load_settings(app: &AppHandle) -> Result<Option<AppSettings>> {
    let path = settings_path(app)?;
    if !path.exists() {
        return Ok(None);
    }

    let raw =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let settings = serde_json::from_str(&raw)
        .with_context(|| format!("failed to parse {}", path.display()))?;
    Ok(Some(settings))
}

fn save_settings(app: &AppHandle, settings: &AppSettings) -> Result<()> {
    let path = settings_path(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    let raw = serde_json::to_string_pretty(settings).context("failed to serialize settings")?;
    fs::write(&path, raw).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

fn system_time_to_ms(value: SystemTime) -> u64 {
    value
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0))
        .as_millis() as u64
}

fn notify(app: &AppHandle, body: &str) {
    let _ = app
        .notification()
        .builder()
        .title("Hutch-Kick")
        .body(body)
        .show();
}

fn set_error_and_stop(app: &AppHandle, shared: &SharedState, message: impl Into<String>) {
    let error_message = message.into();
    {
        let mut inner = shared.lock();
        inner.error_message = Some(error_message.clone());
        let _ = inner.transition(ServiceMachineState::Error);
        inner.push_log(format!("Unexpected failure: {error_message}"));
        let _ = inner.transition(ServiceMachineState::Stopped);
        inner.worker_handle = None;
    }
    notify(app, "Unexpected error. Service stopped.");
}

fn toggle_main_window(app: &AppHandle) -> Result<()> {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible()? {
            window.hide()?;
        } else {
            let _ = position_window_bottom_right(&window);
            window.show()?;
            window.set_focus()?;
        }
    }
    Ok(())
}

fn position_window_bottom_right(window: &WebviewWindow) -> Result<()> {
    let monitor = window
        .current_monitor()?
        .or(window.primary_monitor()?)
        .ok_or_else(|| anyhow!("no monitor available"))?;
    let monitor_pos = monitor.position();
    let monitor_size = monitor.size();
    let window_size = window.outer_size()?;

    let x =
        monitor_pos.x + monitor_size.width as i32 - window_size.width as i32 - WINDOW_MARGIN_X_PX;
    let y =
        monitor_pos.y + monitor_size.height as i32 - window_size.height as i32 - WINDOW_MARGIN_Y_PX;

    window.set_position(Position::Physical(PhysicalPosition::new(x, y)))?;
    Ok(())
}

fn setup_main_window(app: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("main") {
        window.hide()?;
        let _ = position_window_bottom_right(&window);
        let window_clone = window.clone();
        window.on_window_event(move |event| match event {
            WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                let _ = window_clone.hide();
            }
            WindowEvent::Focused(false) => {
                let _ = window_clone.hide();
            }
            _ => {}
        });
    }
    Ok(())
}

fn setup_tray(app: &AppHandle) -> tauri::Result<()> {
    let show_hide = MenuItem::with_id(app, "show-hide", "Show/Hide", true, None::<&str>)?;
    let start_stop = MenuItem::with_id(app, "start-stop", "Start/Stop", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show_hide, &start_stop, &quit])?;

    let mut tray_builder = TrayIconBuilder::with_id("main-tray")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            if event.id() == "show-hide" {
                let _ = toggle_main_window(app);
            } else if event.id() == "start-stop" {
                let app_handle = app.clone();
                tauri::async_runtime::spawn(async move {
                    let shared = app_handle.state::<SharedState>().inner().clone();
                    let current = shared.snapshot().current_state;
                    let result = if current == ServiceMachineState::Running {
                        stop_service_internal(app_handle.clone(), shared).await
                    } else {
                        start_service_internal(app_handle.clone(), shared).await
                    };
                    if let Err(err) = result {
                        set_error_and_stop(
                            &app_handle,
                            &app_handle.state::<SharedState>().inner().clone(),
                            err,
                        );
                    }
                });
            } else if event.id() == "quit" {
                app.exit(0);
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let _ = toggle_main_window(tray.app_handle());
            }
        });

    if let Some(icon) = app.default_window_icon().cloned() {
        tray_builder = tray_builder.icon(icon);
    }

    let _ = tray_builder.build(app)?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn network_connected() -> Result<bool> {
    use std::os::windows::process::CommandExt;

    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let output = Command::new("netsh")
        .args(["interface", "show", "interface"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .context("failed to run netsh")?;

    if !output.status.success() {
        return Err(anyhow!("netsh returned non-zero exit status"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    for raw_line in stdout.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('-') {
            continue;
        }

        let columns: Vec<&str> = line.split_whitespace().collect();
        if columns.len() < 4 {
            continue;
        }

        let state = columns[1].to_ascii_lowercase();
        let interface_type = columns[2].to_ascii_lowercase();
        if state == "connected" && interface_type != "loopback" {
            return Ok(true);
        }
    }

    Ok(false)
}

#[cfg(target_os = "linux")]
fn network_connected() -> Result<bool> {
    let output = Command::new("nmcli")
        .args(["-t", "-f", "TYPE,STATE", "device"])
        .output()
        .context("failed to run nmcli")?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 2 {
                let dev_type = parts[0];
                let state = parts[1];
                if state == "connected" && (dev_type == "wifi" || dev_type == "ethernet") {
                    return Ok(true);
                }
            }
        }
        return Ok(false);
    }

    let output = Command::new("ip")
        .args(["link", "show"])
        .output()
        .context("failed to run ip link")?;

    if !output.status.success() {
        return Err(anyhow!("ip link returned non-zero exit status"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if line.contains("state UP") && !line.contains("lo:") {
            return Ok(true);
        }
    }

    Ok(false)
}

#[cfg(target_os = "macos")]
fn network_connected() -> Result<bool> {
    let output = Command::new("networksetup")
        .args(["-listallhardwareports"])
        .output()
        .context("failed to run networksetup")?;

    if !output.status.success() {
        return Err(anyhow!("networksetup returned non-zero exit status"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut current_device = None;

    for line in stdout.lines() {
        if line.starts_with("Hardware Port:") {
            let port = line.splitn(2, ':').nth(1).unwrap_or("").trim();
            if port.to_lowercase().contains("loopback") {
                current_device = None;
                continue;
            }
        } else if line.starts_with("Device:") {
            current_device = Some(line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string());
        }

        if let Some(ref device) = current_device {
            if !line.starts_with("Hardware") && !line.starts_with("Device") {
                let ifconfig = Command::new("ifconfig")
                    .arg(device)
                    .output()
                    .context("failed to run ifconfig")?;

                if ifconfig.status.success() {
                    let ifout = String::from_utf8_lossy(&ifconfig.stdout);
                    if ifout.contains("status: active") && ifout.contains("inet ") {
                        return Ok(true);
                    }
                }
                current_device = None;
            }
        }
    }

    Ok(false)
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
fn network_connected() -> Result<bool> {
    Err(anyhow!("Network check is not supported on this platform"))
}

#[cfg(target_os = "windows")]
fn check_adapter_internet() -> Result<bool> {
    use std::os::windows::process::CommandExt;

    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let output = Command::new("netsh")
        .args(["interface", "ip", "show", "addresses"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .context("failed to run netsh ip")?;

    if !output.status.success() {
        return Err(anyhow!("netsh ip returned non-zero exit status"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut has_ip = false;
    let mut has_gateway = false;

    for raw_line in stdout.lines() {
        let line = raw_line.trim();
        if line.starts_with("IP Address:") || line.starts_with("IP Address(b):") {
            let addr = line.splitn(2, ':').nth(1).unwrap_or("").trim();
            if !addr.is_empty() && addr != "0.0.0.0" {
                has_ip = true;
            }
        }
        if line.starts_with("Default Gateway:") || line.starts_with("Default Gateway(b):") {
            let gw = line.splitn(2, ':').nth(1).unwrap_or("").trim();
            if !gw.is_empty() && gw != "0.0.0.0" {
                has_gateway = true;
            }
        }
    }

    Ok(has_ip && has_gateway)
}

#[cfg(target_os = "linux")]
fn check_adapter_internet() -> Result<bool> {
    let output = Command::new("ip")
        .args(["route", "show", "default"])
        .output()
        .context("failed to run ip route")?;

    if !output.status.success() {
        return Err(anyhow!("ip route returned non-zero exit status"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let has_default_route = stdout.lines().any(|line| line.contains("default"));

    let output = Command::new("ip")
        .args(["addr", "show"])
        .output()
        .context("failed to run ip addr")?;

    if !output.status.success() {
        return Err(anyhow!("ip addr returned non-zero exit status"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut has_ip = false;

    for line in stdout.lines() {
        if line.trim().starts_with("inet ") && !line.contains("127.0.0.1") {
            has_ip = true;
            break;
        }
    }

    Ok(has_ip && has_default_route)
}

#[cfg(target_os = "macos")]
fn check_adapter_internet() -> Result<bool> {
    let output = Command::new("netstat")
        .args(["-nr", "inet"])
        .output()
        .context("failed to run netstat")?;

    if !output.status.success() {
        return Err(anyhow!("netstat returned non-zero exit status"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let has_default_route = stdout.lines().any(|line| line.contains("default"));

    let output = Command::new("ifconfig")
        .args(["inet"])
        .output()
        .context("failed to run ifconfig inet")?;

    if !output.status.success() {
        return Err(anyhow!("ifconfig inet returned non-zero exit status"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let has_ip = stdout.lines().any(|line| line.trim().starts_with("inet ") && !line.contains("127.0.0.1"));

    Ok(has_ip && has_default_route)
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
fn check_adapter_internet() -> Result<bool> {
    Err(anyhow!("Internet check is not supported on this platform"))
}

async fn internet_online(client: &Client) -> bool {
    if let Ok(connected) = check_adapter_internet() {
        if connected {
            return true;
        }
    }

    let result = client
        .head(CONNECTIVITY_URL)
        .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECONDS))
        .send()
        .await;

    match result {
        Ok(response) => response.status().is_success() || response.status().is_redirection(),
        Err(_) => false,
    }
}

async fn refresh_connectivity_state(shared: &SharedState) {
    let wifi_status = match network_connected() {
        Ok(true) => WifiStatus::Connected,
        Ok(false) => WifiStatus::Disconnected,
        Err(_) => WifiStatus::Unknown,
    };

    let client = {
        let mut inner = shared.lock();
        if service_owns_connectivity(inner.current_state) {
            return;
        }
        inner.wifi_status = wifi_status;
        if wifi_status != WifiStatus::Connected {
            inner.internet_status = InternetStatus::Unknown;
            return;
        }
        inner.client.clone()
    };

    let internet_status = if internet_online(&client).await {
        InternetStatus::Online
    } else {
        InternetStatus::Offline
    };

    let mut inner = shared.lock();
    if service_owns_connectivity(inner.current_state) {
        return;
    }
    if inner.wifi_status == WifiStatus::Connected {
        inner.internet_status = internet_status;
    }
}

async fn connectivity_monitor_loop(shared: SharedState) {
    refresh_connectivity_state(&shared).await;

    loop {
        tokio::time::sleep(Duration::from_secs(CONNECTIVITY_MONITOR_INTERVAL_SECONDS)).await;
        refresh_connectivity_state(&shared).await;
    }
}

async fn kick(client: &Client) -> Result<Duration> {
    let started_at = Instant::now();
    let result = client
        .get(KICK_URL)
        .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECONDS))
        .send()
        .await;

    match result {
        Ok(_) => Ok(started_at.elapsed()),
        Err(err) => Err(anyhow!(err)),
    }
}

fn stop_for_network_disconnect(app: &AppHandle, shared: &SharedState) {
    {
        let mut inner = shared.lock();
        if inner.current_state != ServiceMachineState::Running {
            return;
        }

        inner.wifi_status = WifiStatus::Disconnected;
        inner.internet_status = InternetStatus::Unknown;

        if inner.transition(ServiceMachineState::Stopping).is_err() {
            drop(inner);
            set_error_and_stop(app, shared, "Failed transition to STOPPING");
            return;
        }

        inner.push_log("Network adapter disconnected while running.");
        inner.worker_handle = None;

        if inner.transition(ServiceMachineState::Stopped).is_err() {
            drop(inner);
            set_error_and_stop(app, shared, "Failed transition to STOPPED");
            return;
        }
    }

    notify(app, "Network disconnected. Service stopped.");
}

async fn worker_loop(app: AppHandle, shared: SharedState) {
    let mut consecutive_kick_failures = 0u32;
    const MAX_KICK_RETRIES: u32 = 3;

    loop {
        let (client, interval_seconds, current_state) = {
            let inner = shared.lock();
            (
                inner.client.clone(),
                sanitize_interval_seconds(inner.interval_seconds),
                inner.current_state,
            )
        };

        if current_state != ServiceMachineState::Running {
            break;
        }

        match network_connected() {
            Ok(true) => {
                let mut inner = shared.lock();
                inner.wifi_status = WifiStatus::Connected;
            }
            Ok(false) => {
                stop_for_network_disconnect(&app, &shared);
                break;
            }
            Err(_) => {
                stop_for_network_disconnect(&app, &shared);
                break;
            }
        }

        let internet_ok = internet_online(&client).await;
        {
            let mut inner = shared.lock();
            let previous_status = inner.internet_status;
            inner.internet_status = if internet_ok {
                InternetStatus::Online
            } else {
                InternetStatus::Offline
            };

            match (previous_status, inner.internet_status) {
                (_, InternetStatus::Offline) if previous_status != InternetStatus::Offline => {
                    inner.push_log("Internet connectivity lost. Waiting for reconnection.");
                }
                (InternetStatus::Offline, InternetStatus::Online) => {
                    inner.push_log("Internet connectivity restored.");
                }
                _ => {}
            }
        }

        if !internet_ok {
            tokio::time::sleep(Duration::from_secs(interval_seconds)).await;
            continue;
        }

        let latency = match kick(&client).await {
            Ok(latency) => {
                consecutive_kick_failures = 0;
                Ok(latency)
            }
            Err(err) => {
                consecutive_kick_failures += 1;
                if consecutive_kick_failures >= MAX_KICK_RETRIES {
                    Err(err)
                } else {
                    {
                        let mut inner = shared.lock();
                        inner.push_log(format!(
                            "Kick failed (attempt {}/{}), retrying...",
                            consecutive_kick_failures, MAX_KICK_RETRIES
                        ));
                    }
                    tokio::time::sleep(Duration::from_secs(interval_seconds)).await;
                    continue;
                }
            }
        };

        let latency = match latency {
            Ok(latency) => latency,
            Err(_) => {
                set_error_and_stop(&app, &shared, "Kick request failed after retries.");
                break;
            }
        };

        {
            let mut inner = shared.lock();
            inner.last_kick_time = Some(SystemTime::now());
            inner.push_log(format!("Kick latency: {}", format_latency(latency)));
        }

        tokio::time::sleep(Duration::from_secs(interval_seconds)).await;
    }
}

async fn start_service_internal(
    app: AppHandle,
    shared: SharedState,
) -> Result<ServiceSnapshot, String> {
    {
        let mut inner = shared.lock();
        if inner.current_state != ServiceMachineState::Stopped {
            return Ok(inner.snapshot());
        }

        inner
            .transition(ServiceMachineState::Starting)
            .map_err(|err| err.to_string())?;
        inner.error_message = None;
    }

    let network_connected = match network_connected() {
        Ok(connected) => connected,
        Err(err) => {
            {
                let mut inner = shared.lock();
                inner.wifi_status = WifiStatus::Unknown;
                inner.error_message = Some("Network state unknown. Start blocked.".to_string());
                inner.push_log(format!("Network check failed: {err}"));
                let _ = inner.transition(ServiceMachineState::Stopped);
            }
            notify(&app, "Network state unknown. Check Wi-Fi or Ethernet.");
            return Ok(shared.snapshot());
        }
    };

    if !network_connected {
        {
            let mut inner = shared.lock();
            inner.wifi_status = WifiStatus::Disconnected;
            inner.internet_status = InternetStatus::Unknown;
            inner.error_message =
                Some("No active network adapter. Connect Wi-Fi or Ethernet to start.".to_string());
            inner.push_log("Start blocked because no network adapter is connected.");
            let _ = inner.transition(ServiceMachineState::Stopped);
        }
        notify(&app, "Connect Wi-Fi or Ethernet to start.");
        return Ok(shared.snapshot());
    }

    {
        let mut inner = shared.lock();
        inner.wifi_status = WifiStatus::Connected;
        inner.error_message = None;
        inner
            .transition(ServiceMachineState::Running)
            .map_err(|err| err.to_string())?;
    }

    let app_for_worker = app.clone();
    let shared_for_worker = shared.clone();
    let handle = tauri::async_runtime::spawn(async move {
        worker_loop(app_for_worker, shared_for_worker).await;
    });

    {
        let mut inner = shared.lock();
        if inner.current_state == ServiceMachineState::Running {
            inner.worker_handle = Some(handle);
        } else {
            drop(inner);
            handle.abort();
        }
    }

    Ok(shared.snapshot())
}

async fn stop_service_internal(
    _app: AppHandle,
    shared: SharedState,
) -> Result<ServiceSnapshot, String> {
    let handle = {
        let mut inner = shared.lock();
        if inner.current_state != ServiceMachineState::Running {
            return Ok(inner.snapshot());
        }

        inner
            .transition(ServiceMachineState::Stopping)
            .map_err(|err| err.to_string())?;
        inner.worker_handle.take()
    };

    if let Some(worker) = handle {
        worker.abort();
    }

    {
        let mut inner = shared.lock();
        if inner.current_state == ServiceMachineState::Stopping {
            inner
                .transition(ServiceMachineState::Stopped)
                .map_err(|err| err.to_string())?;
            inner.worker_handle = None;
            inner.error_message = None;
        }
    }

    Ok(shared.snapshot())
}

#[tauri::command]
fn get_status(state: State<'_, SharedState>) -> ServiceSnapshot {
    state.inner().snapshot()
}

#[tauri::command]
async fn start_service(
    app: AppHandle,
    state: State<'_, SharedState>,
) -> Result<ServiceSnapshot, String> {
    start_service_internal(app, state.inner().clone()).await
}

#[tauri::command]
async fn stop_service(
    app: AppHandle,
    state: State<'_, SharedState>,
) -> Result<ServiceSnapshot, String> {
    stop_service_internal(app, state.inner().clone()).await
}

#[tauri::command]
async fn kick_now(
    app: AppHandle,
    state: State<'_, SharedState>,
) -> Result<ServiceSnapshot, String> {
    let client = {
        let inner = state.inner().lock();
        if inner.current_state != ServiceMachineState::Running {
            return Ok(inner.snapshot());
        }
        inner.client.clone()
    };

    let latency = match kick(&client).await {
        Ok(latency) => latency,
        Err(_) => {
            set_error_and_stop(&app, state.inner(), "Manual kick failed.");
            return Ok(state.inner().snapshot());
        }
    };

    {
        let mut inner = state.inner().lock();
        inner.last_kick_time = Some(SystemTime::now());
        inner.push_log(format!("Manual kick latency: {}", format_latency(latency)));
    }

    Ok(state.inner().snapshot())
}

#[tauri::command]
fn set_interval(
    app: AppHandle,
    interval_seconds: u64,
    state: State<'_, SharedState>,
) -> ServiceSnapshot {
    let mut inner = state.inner().lock();
    let sanitized = sanitize_interval_seconds(interval_seconds);
    inner.interval_seconds = sanitized;

    if let Err(err) = save_settings(
        &app,
        &AppSettings {
            interval_seconds: sanitized,
        },
    ) {
        inner.push_log(format!("Failed to save interval: {err}"));
    }

    inner.snapshot()
}

#[tauri::command]
fn quit_app(app: AppHandle) {
    app.exit(0);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let client = Client::builder()
        .connect_timeout(Duration::from_secs(CONNECT_TIMEOUT_SECONDS))
        .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECONDS))
        .build()
        .expect("failed to create HTTP client");

    tauri::Builder::default()
        .manage(SharedState::new(client))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            setup_main_window(app.handle())?;
            setup_tray(app.handle())?;
            let shared = app.state::<SharedState>().inner().clone();
            match load_settings(app.handle()) {
                Ok(Some(settings)) => {
                    shared.lock().interval_seconds =
                        sanitize_interval_seconds(settings.interval_seconds);
                }
                Ok(None) => {}
                Err(err) => {
                    shared
                        .lock()
                        .push_log(format!("Failed to load settings: {err}"));
                }
            }
            let shared = app.state::<SharedState>().inner().clone();
            tauri::async_runtime::spawn(async move {
                connectivity_monitor_loop(shared).await;
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_status,
            start_service,
            stop_service,
            kick_now,
            set_interval,
            quit_app
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
