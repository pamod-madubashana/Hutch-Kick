import { Wifi, WifiOff, Globe, Clock } from "lucide-react";
import type { InternetStatus, WifiStatus } from "@/hooks/useServiceState";

interface StatusCardProps {
  wifiStatus: WifiStatus;
  internetStatus: InternetStatus;
  lastKick: Date | null;
}

function timeAgo(date: Date | null): string {
  if (!date) return "Never";
  const seconds = Math.floor((Date.now() - date.getTime()) / 1000);
  if (seconds < 60) return `${seconds}s ago`;
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes}m ago`;
  return `${Math.floor(minutes / 60)}h ago`;
}

export function StatusCard({ wifiStatus, internetStatus, lastKick }: StatusCardProps) {
  const networkConnected = wifiStatus === "CONNECTED";
  const internetConnected = internetStatus === "ONLINE";

  const rows = [
    {
      icon: networkConnected ? Wifi : WifiOff,
      label: "Network",
      value: wifiStatus === "UNKNOWN" ? "Unknown" : networkConnected ? "Connected" : "Disconnected",
      toneClass:
        wifiStatus === "UNKNOWN"
          ? "text-status-warn"
          : networkConnected
            ? "text-status-good"
            : "text-status-bad",
    },
    {
      icon: Globe,
      label: "Internet",
      value: internetStatus === "UNKNOWN" ? "Unknown" : internetConnected ? "Connected" : "Disconnected",
      toneClass:
        internetStatus === "UNKNOWN"
          ? "text-status-warn"
          : internetConnected
            ? "text-status-good"
            : "text-status-bad",
    },
    {
      icon: Clock,
      label: "Last Kick",
      value: timeAgo(lastKick),
      toneClass: "text-foreground",
    },
  ];

  return (
    <div className="mx-3 rounded-xl glass-surface p-3 shadow-[0_16px_40px_-32px_rgba(0,0,0,0.9)] space-y-2.5">
      <div className="flex items-center justify-between">
        <span className="text-[10px] font-semibold uppercase tracking-[0.18em] text-muted-foreground/75">
          Connection Health
        </span>
        <span className="rounded-full border border-border/50 bg-secondary/25 px-2 py-0.5 text-[9px] font-medium uppercase tracking-[0.16em] text-muted-foreground/80">
          Live
        </span>
      </div>
      {rows.map((row) => (
        <div key={row.label} className="flex items-center justify-between rounded-lg bg-secondary/18 px-2.5 py-2 text-xs">
          <div className="flex items-center gap-2 text-muted-foreground">
            <row.icon className="h-3.5 w-3.5" />
            <span>{row.label}</span>
          </div>
          <span className={`font-medium ${row.toneClass}`}>
            {row.value}
          </span>
        </div>
      ))}
      <p className="border-t border-border/40 pt-1 text-[10px] text-muted-foreground">
        Service stops automatically if network or internet drops.
      </p>
    </div>
  );
}
