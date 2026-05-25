import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useState } from "react";

export type ServiceStatus = "RUNNING" | "STOPPED" | "STARTING" | "STOPPING" | "ERROR";
export type WifiStatus = "CONNECTED" | "DISCONNECTED" | "UNKNOWN";
export type InternetStatus = "ONLINE" | "OFFLINE" | "UNKNOWN";
export type KickInterval = "5" | "10" | "15" | "20" | "25" | "30";

export interface LogEntry {
  id: string;
  message: string;
  timestamp: Date;
}

interface BackendLogEntry {
  id: number;
  message: string;
  timestampMs: number;
}

interface BackendSnapshot {
  currentState: ServiceStatus;
  wifiStatus: WifiStatus;
  internetStatus: InternetStatus;
  lastKickTimeMs: number | null;
  intervalSeconds: number;
  logs: BackendLogEntry[];
  errorMessage: string | null;
}

interface ServiceState {
  status: ServiceStatus;
  wifiStatus: WifiStatus;
  internetStatus: InternetStatus;
  lastKick: Date | null;
  kickInterval: KickInterval;
  logs: LogEntry[];
  errorMessage: string | null;
  backendConnected: boolean;
}

const POLL_INTERVAL_MS = 1200;

const initialState: ServiceState = {
  status: "STOPPED",
  wifiStatus: "UNKNOWN",
  internetStatus: "UNKNOWN",
  lastKick: null,
  kickInterval: "20",
  logs: [],
  errorMessage: null,
  backendConnected: true,
};

function toKickInterval(seconds: number): KickInterval {
  switch (seconds) {
    case 5:
      return "5";
    case 10:
      return "10";
    case 15:
      return "15";
    case 20:
      return "20";
    case 25:
      return "25";
    case 30:
      return "30";
    default:
      return "20";
  }
}

function mapError(err: unknown): string {
  if (err instanceof Error) return err.message;
  return String(err);
}

export function useServiceState() {
  const [state, setState] = useState<ServiceState>(initialState);

  const applySnapshot = useCallback((snapshot: BackendSnapshot) => {
    setState((prev) => ({
      ...prev,
      status: snapshot.currentState,
      wifiStatus: snapshot.wifiStatus,
      internetStatus: snapshot.internetStatus,
      lastKick: snapshot.lastKickTimeMs ? new Date(snapshot.lastKickTimeMs) : null,
      kickInterval: toKickInterval(snapshot.intervalSeconds),
      logs: snapshot.logs.map((entry) => ({
        id: String(entry.id),
        message: entry.message,
        timestamp: new Date(entry.timestampMs),
      })),
      errorMessage: snapshot.errorMessage,
      backendConnected: true,
    }));
  }, []);

  const refreshStatus = useCallback(async () => {
    try {
      const snapshot = await invoke<BackendSnapshot>("get_status");
      applySnapshot(snapshot);
    } catch {
      setState((prev) => ({
        ...prev,
        backendConnected: false,
      }));
    }
  }, [applySnapshot]);

  useEffect(() => {
    void refreshStatus();
    const timer = window.setInterval(() => {
      void refreshStatus();
    }, POLL_INTERVAL_MS);

    return () => window.clearInterval(timer);
  }, [refreshStatus]);

  const startService = useCallback(async () => {
    try {
      const snapshot = await invoke<BackendSnapshot>("start_service");
      applySnapshot(snapshot);
    } catch (err) {
      setState((prev) => ({ ...prev, errorMessage: mapError(err) }));
    }
  }, [applySnapshot]);

  const stopService = useCallback(async () => {
    try {
      const snapshot = await invoke<BackendSnapshot>("stop_service");
      applySnapshot(snapshot);
    } catch (err) {
      setState((prev) => ({ ...prev, errorMessage: mapError(err) }));
    }
  }, [applySnapshot]);

  const kickNow = useCallback(async () => {
    try {
      const snapshot = await invoke<BackendSnapshot>("kick_now");
      applySnapshot(snapshot);
    } catch (err) {
      setState((prev) => ({ ...prev, errorMessage: mapError(err) }));
    }
  }, [applySnapshot]);

  const setKickInterval = useCallback(async (value: KickInterval) => {
    const intervalSeconds = Number(value);
    try {
      const snapshot = await invoke<BackendSnapshot>("set_interval", { intervalSeconds });
      applySnapshot(snapshot);
    } catch (err) {
      setState((prev) => ({ ...prev, errorMessage: mapError(err) }));
    }
  }, [applySnapshot]);

  const quitApp = useCallback(async () => {
    try {
      await invoke("quit_app");
    } catch (err) {
      setState((prev) => ({ ...prev, errorMessage: mapError(err) }));
    }
  }, []);

  return {
    ...state,
    startService,
    stopService,
    kickNow,
    setKickInterval,
    quitApp,
  };
}
