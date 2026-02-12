use anyhow::{anyhow, Context, Result};
use reqwest::Client;
use serde::Serialize;
use std::{
    collections::VecDeque,
    process::Command,
    sync::{Arc, Mutex, MutexGuard},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, PhysicalPosition, Position, State, WebviewWindow, WindowEvent,
};
use tauri_plugin_notification::NotificationExt;

const MIN_INTERVAL_SECONDS: u64 = 20;
const DEFAULT_INTERVAL_SECONDS: u64 = 20;
const CONNECT_TIMEOUT_SECONDS: u64 = 3;
const REQUEST_TIMEOUT_SECONDS: u64 = 5;
const MAX_LOGS: usize = 30;
const WINDOW_MARGIN_PX: i32 = 16;
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
        let mut state = Self {
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
        state.push_log("Service initialized in STOPPED state.");
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
        self.0.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
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

fn now_ms() -> u64 {
    system_time_to_ms(SystemTime::now())
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
        .title("HutchKick")
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
        inner.push_log("Service moved to STOPPED after ERROR.");
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

    let x = monitor_pos.x + monitor_size.width as i32 - window_size.width as i32 - WINDOW_MARGIN_PX;
    let y = monitor_pos.y + monitor_size.height as i32 - window_size.height as i32 - WINDOW_MARGIN_PX;

    window.set_position(Position::Physical(PhysicalPosition::new(x, y)))?;
    Ok(())
}

fn setup_main_window(app: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("main") {
        window.hide()?;
        let _ = position_window_bottom_right(&window);
        let window_clone = window.clone();
        window.on_window_event(move |event| {
            match event {
                WindowEvent::CloseRequested { api, .. } => {
                    api.prevent_close();
                    let _ = window_clone.hide();
                }
                WindowEvent::Focused(false) => {
                    let _ = window_clone.hide();
                }
                _ => {}
            }
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
                        set_error_and_stop(&app_handle, &app_handle.state::<SharedState>().inner().clone(), err);
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
fn wifi_connected_windows() -> Result<bool> {
    use std::os::windows::process::CommandExt;

    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let output = Command::new("netsh")
        .args(["wlan", "show", "interfaces"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .context("failed to run netsh")?;

    if !output.status.success() {
        return Err(anyhow!("netsh returned non-zero exit status"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    for raw_line in stdout.lines() {
        let line = raw_line.trim().to_ascii_lowercase();
        if line.starts_with("state") {
            let mut parts = line.splitn(2, ':');
            let _ = parts.next();
            if let Some(value) = parts.next() {
                return Ok(value.trim().contains("connected"));
            }
        }
    }

    Err(anyhow!("unable to parse Wi-Fi state from netsh output"))
}

#[cfg(not(target_os = "windows"))]
fn wifi_connected_windows() -> Result<bool> {
    Err(anyhow!("Wi-Fi check is only supported on Windows"))
}

async fn internet_online(client: &Client) -> bool {
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

async fn kick(client: &Client) -> bool {
    let result = client
        .get(KICK_URL)
        .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECONDS))
        .send()
        .await;
    result.is_ok()
}

fn stop_for_connectivity(app: &AppHandle, shared: &SharedState, wifi_lost: bool) {
    {
        let mut inner = shared.lock();
        if inner.current_state != ServiceMachineState::Running {
            return;
        }

        if wifi_lost {
            inner.wifi_status = WifiStatus::Disconnected;
            inner.internet_status = InternetStatus::Unknown;
        } else {
            inner.internet_status = InternetStatus::Offline;
        }

        if inner.transition(ServiceMachineState::Stopping).is_err() {
            drop(inner);
            set_error_and_stop(app, shared, "Failed transition to STOPPING");
            return;
        }

        if wifi_lost {
            inner.push_log("Wi-Fi disconnected while running.");
        } else {
            inner.push_log("Internet connectivity lost while running.");
        }
        inner.worker_handle = None;

        if inner.transition(ServiceMachineState::Stopped).is_err() {
            drop(inner);
            set_error_and_stop(app, shared, "Failed transition to STOPPED");
            return;
        }
        inner.push_log("Service moved to STOPPED.");
    }

    if wifi_lost {
        notify(app, "Wi-Fi disconnected. Service stopped.");
    } else {
        notify(app, "Internet lost. Service stopped.");
    }
}

async fn worker_loop(app: AppHandle, shared: SharedState) {
    loop {
        let (client, interval_seconds, current_state) = {
            let inner = shared.lock();
            (
                inner.client.clone(),
                inner.interval_seconds.max(MIN_INTERVAL_SECONDS),
                inner.current_state,
            )
        };

        if current_state != ServiceMachineState::Running {
            break;
        }

        match wifi_connected_windows() {
            Ok(true) => {
                let mut inner = shared.lock();
                inner.wifi_status = WifiStatus::Connected;
            }
            Ok(false) => {
                stop_for_connectivity(&app, &shared, true);
                break;
            }
            Err(_) => {
                stop_for_connectivity(&app, &shared, true);
                break;
            }
        }

        let internet_ok = internet_online(&client).await;
        {
            let mut inner = shared.lock();
            inner.internet_status = if internet_ok {
                InternetStatus::Online
            } else {
                InternetStatus::Offline
            };
        }

        if !internet_ok {
            stop_for_connectivity(&app, &shared, false);
            break;
        }

        if !kick(&client).await {
            set_error_and_stop(&app, &shared, "Kick request failed.");
            break;
        }

        {
            let mut inner = shared.lock();
            inner.last_kick_time = Some(SystemTime::now());
            inner.push_log("Kick sent successfully.");
        }

        tokio::time::sleep(Duration::from_secs(interval_seconds)).await;
    }
}

async fn start_service_internal(app: AppHandle, shared: SharedState) -> Result<ServiceSnapshot, String> {
    {
        let mut inner = shared.lock();
        if inner.current_state != ServiceMachineState::Stopped {
            return Ok(inner.snapshot());
        }

        inner
            .transition(ServiceMachineState::Starting)
            .map_err(|err| err.to_string())?;
        inner.error_message = None;
        inner.push_log("Start requested.");
    }

    let wifi_connected = match wifi_connected_windows() {
        Ok(connected) => connected,
        Err(err) => {
            {
                let mut inner = shared.lock();
                inner.wifi_status = WifiStatus::Unknown;
                inner.error_message = Some("Wi-Fi state unknown. Start blocked.".to_string());
                inner.push_log(format!("Wi-Fi check failed: {err}"));
                let _ = inner.transition(ServiceMachineState::Stopped);
            }
            notify(&app, "Wi-Fi is off. Turn it on to start.");
            return Ok(shared.snapshot());
        }
    };

    if !wifi_connected {
        {
            let mut inner = shared.lock();
            inner.wifi_status = WifiStatus::Disconnected;
            inner.internet_status = InternetStatus::Unknown;
            inner.error_message = Some("Wi-Fi is off. Turn it on to start.".to_string());
            inner.push_log("Start blocked because Wi-Fi is disconnected.");
            let _ = inner.transition(ServiceMachineState::Stopped);
        }
        notify(&app, "Wi-Fi is off. Turn it on to start.");
        return Ok(shared.snapshot());
    }

    {
        let mut inner = shared.lock();
        inner.wifi_status = WifiStatus::Connected;
    }

    let client = {
        let inner = shared.lock();
        inner.client.clone()
    };

    let internet_ok = internet_online(&client).await;
    if !internet_ok {
        let mut inner = shared.lock();
        inner.internet_status = InternetStatus::Offline;
        inner.error_message = Some("Internet unavailable. Start blocked.".to_string());
        inner.push_log("Start blocked because internet is offline.");
        let _ = inner.transition(ServiceMachineState::Stopped);
        return Ok(inner.snapshot());
    }

    {
        let mut inner = shared.lock();
        inner.internet_status = InternetStatus::Online;
        inner
            .transition(ServiceMachineState::Running)
            .map_err(|err| err.to_string())?;
        inner.push_log("Service transitioned to RUNNING.");
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

async fn stop_service_internal(_app: AppHandle, shared: SharedState) -> Result<ServiceSnapshot, String> {
    let handle = {
        let mut inner = shared.lock();
        if inner.current_state != ServiceMachineState::Running {
            return Ok(inner.snapshot());
        }

        inner
            .transition(ServiceMachineState::Stopping)
            .map_err(|err| err.to_string())?;
        inner.push_log("Stop requested by user.");
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
            inner.push_log("Service stopped by user.");
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
async fn kick_now(app: AppHandle, state: State<'_, SharedState>) -> Result<ServiceSnapshot, String> {
    let client = {
        let inner = state.inner().lock();
        if inner.current_state != ServiceMachineState::Running {
            return Ok(inner.snapshot());
        }
        inner.client.clone()
    };

    if !kick(&client).await {
        set_error_and_stop(&app, state.inner(), "Manual kick failed.");
        return Ok(state.inner().snapshot());
    }

    {
        let mut inner = state.inner().lock();
        inner.last_kick_time = Some(SystemTime::now());
        inner.push_log("Manual kick sent successfully.");
    }

    Ok(state.inner().snapshot())
}

#[tauri::command]
fn set_interval(interval_seconds: u64, state: State<'_, SharedState>) -> ServiceSnapshot {
    let mut inner = state.inner().lock();
    let sanitized = interval_seconds.max(MIN_INTERVAL_SECONDS);
    inner.interval_seconds = sanitized;
    inner.push_log(format!("Kick interval set to {sanitized}s."));
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
