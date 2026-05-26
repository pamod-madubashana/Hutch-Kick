import { useId } from "react";
import { Wifi, WifiOff, Globe, Clock } from "lucide-react";
import type { InternetStatus, LogEntry, WifiStatus } from "@/hooks/useServiceState";

const LATENCY_WINDOW_MS = 60_000;
const DEFAULT_LATENCY_SCALE_MAX = 500;

interface StatusCardProps {
  wifiStatus: WifiStatus;
  internetStatus: InternetStatus;
  lastKick: Date | null;
  logs: LogEntry[];
}

interface LatencyPoint {
  x: number;
  y: number;
  latencyMs: number;
}

function timeAgo(date: Date | null): string {
  if (!date) return "Never";
  const seconds = Math.floor((Date.now() - date.getTime()) / 1000);
  if (seconds < 60) return `${seconds}s ago`;
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes}m ago`;
  return `${Math.floor(minutes / 60)}h ago`;
}

function extractLatencySamples(logs: LogEntry[]) {
  return logs
    .map((entry) => {
      const match = entry.message.match(/(?:manual\s+)?kick latency:\s*(\d+(?:\.\d+)?)ms/i);

      if (!match) return null;

      return {
        timestampMs: entry.timestamp.getTime(),
        latencyMs: Number(match[1]),
      };
    })
    .filter((sample): sample is { timestampMs: number; latencyMs: number } => sample !== null)
    .sort((left, right) => left.timestampMs - right.timestampMs);
}

function buildLatencyPoints(logs: LogEntry[], nowMs: number) {
  const samples = extractLatencySamples(logs);
  const chartEndMs = samples.at(-1)?.timestampMs ?? nowMs;
  const chartStartMs = chartEndMs - LATENCY_WINDOW_MS;
  const windowedSamples = samples.filter((sample) => sample.timestampMs >= chartStartMs);
  const latencyScaleMax = Math.max(
    DEFAULT_LATENCY_SCALE_MAX,
    Math.ceil((Math.max(...windowedSamples.map((sample) => sample.latencyMs), DEFAULT_LATENCY_SCALE_MAX) + 25) / 100) * 100,
  );

  const points = windowedSamples.map<LatencyPoint>((sample) => {
    const x = ((sample.timestampMs - chartStartMs) / LATENCY_WINDOW_MS) * 100;
    const y = 100 - (Math.min(sample.latencyMs, latencyScaleMax) / latencyScaleMax) * 100;

    return {
      x: Math.min(100, Math.max(0, x)),
      y: Math.min(100, Math.max(0, y)),
      latencyMs: sample.latencyMs,
    };
  });

  return { points, latencyScaleMax };
}

function buildSmoothLinePath(points: LatencyPoint[]) {
  if (points.length === 0) return "";
  if (points.length === 1) return `M ${points[0].x} ${points[0].y}`;

  let path = `M ${points[0].x} ${points[0].y}`;

  for (let index = 1; index < points.length; index += 1) {
    const previous = points[index - 1];
    const current = points[index];
    const controlX = (previous.x + current.x) / 2;

    path += ` C ${controlX} ${previous.y}, ${controlX} ${current.y}, ${current.x} ${current.y}`;
  }

  return path;
}

function buildAreaPath(points: LatencyPoint[]) {
  if (points.length < 2) return "";

  const linePath = buildSmoothLinePath(points);
  return `${linePath} L ${points[points.length - 1].x} 100 L ${points[0].x} 100 Z`;
}

function getLatencyToneClass(latencyMs: number) {
  if (latencyMs < 70) return "text-status-good";
  if (latencyMs <= 100) return "text-status-warn";
  if (latencyMs <= 200) return "text-status-alert";
  return "text-status-bad";
}

export function StatusCard({ wifiStatus, internetStatus, lastKick, logs }: StatusCardProps) {
  const networkConnected = wifiStatus === "CONNECTED";
  const internetConnected = internetStatus === "ONLINE";
  const chartId = useId();
  const nowMs = Date.now();
  const { points, latencyScaleMax } = buildLatencyPoints(logs, nowMs);
  const linePath = buildSmoothLinePath(points);
  const areaPath = buildAreaPath(points);

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
    <div className="mx-3 rounded-[18px] border border-white/10 bg-[radial-gradient(circle_at_top_left,rgba(79,255,155,0.13),transparent_42%),radial-gradient(circle_at_bottom_right,rgba(37,255,115,0.08),transparent_38%),linear-gradient(180deg,rgba(255,255,255,0.03),rgba(255,255,255,0.01))] p-2 shadow-[0_24px_64px_-40px_rgba(10,255,122,0.45)] backdrop-blur-xl">
      <div className="flex items-center justify-between">
        <span className="text-[10px] font-semibold uppercase tracking-[0.28em] text-foreground/90">
          Connection Health
        </span>
        <span className="rounded-full border border-white/10 bg-black/15 px-2.5 py-1 text-[9px] font-medium uppercase tracking-[0.24em] text-muted-foreground/90 shadow-[0_0_24px_rgba(41,255,120,0.08)]">
          <span className="mr-1.5 inline-block h-2 w-2 rounded-full bg-[hsl(var(--status-good))] shadow-[0_0_10px_rgba(52,211,153,0.9)]" />
          Live
        </span>
      </div>

      <p className="mt-1 text-[8px] uppercase tracking-[0.22em] text-muted-foreground/70">
        Ping / Latency (ms)
      </p>

      <div className="relative mt-1 h-[112px] overflow-hidden rounded-[14px] border border-white/6 bg-black/12 px-2 py-1.5">
        <svg className="pointer-events-none absolute inset-0 h-full w-full" viewBox="0 0 100 100" preserveAspectRatio="none" aria-hidden="true">
          <defs>
            <linearGradient id={`${chartId}-area`} x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stopColor="rgba(52, 211, 153, 0.26)" />
              <stop offset="100%" stopColor="rgba(52, 211, 153, 0)" />
            </linearGradient>
            <linearGradient id={`${chartId}-line`} x1="0" y1="0" x2="1" y2="0">
              <stop offset="0%" stopColor="rgba(34, 197, 94, 0.78)" />
              <stop offset="70%" stopColor="rgba(132, 204, 22, 0.92)" />
              <stop offset="100%" stopColor="rgba(250, 204, 21, 0.88)" />
            </linearGradient>
            <filter id={`${chartId}-glow`}>
              <feGaussianBlur stdDeviation="1.4" result="blur" />
              <feMerge>
                <feMergeNode in="blur" />
                <feMergeNode in="SourceGraphic" />
              </feMerge>
            </filter>
          </defs>

          {[18, 36, 54, 72, 90].map((y) => (
            <line
              key={`h-${y}`}
              x1="4"
              y1={y}
              x2="96"
              y2={y}
              style={{ stroke: "hsl(var(--border) / 0.16)", strokeDasharray: "3 4" }}
            />
          ))}
          {[12, 36, 60, 84].map((x) => (
            <line
              key={`v-${x}`}
              x1={x}
              y1="14"
              x2={x}
              y2="92"
              style={{ stroke: "hsl(var(--border) / 0.12)", strokeDasharray: "4 5" }}
            />
          ))}

          {areaPath ? <path d={areaPath} fill={`url(#${chartId}-area)`} opacity="0.95" /> : null}
          {points.length > 1 ? (
            <path
              d={linePath}
              fill="none"
              stroke={`url(#${chartId}-line)`}
              strokeWidth="1.15"
              strokeLinecap="round"
              strokeLinejoin="round"
              filter={`url(#${chartId}-glow)`}
            />
          ) : null}
          {points.map((point, index) => (
            <circle
              key={`${point.x}-${point.y}-${index}`}
              cx={point.x}
              cy={point.y}
              r={index === points.length - 1 ? 1.6 : 1}
              className={getLatencyToneClass(point.latencyMs)}
              style={{ fill: "currentColor", filter: index === points.length - 1 ? "drop-shadow(0 0 10px currentColor)" : undefined }}
            />
          ))}
        </svg>

        <div className="pointer-events-none absolute left-2 top-1.5 text-[7px] font-medium text-muted-foreground/38">
          {latencyScaleMax}
        </div>
        <div className="pointer-events-none absolute left-2 bottom-1 text-[7px] font-medium text-muted-foreground/38">
          -60s
        </div>
        <div className="pointer-events-none absolute right-2 bottom-1 text-[7px] font-medium text-muted-foreground/38">
          Now
        </div>

        <div className="relative z-10 flex h-full flex-col justify-center gap-2 px-1 py-3">
          {rows.map((row) => (
            <div key={row.label} className="flex items-center justify-between gap-3 text-xs">
              <div className="flex items-center gap-2 text-muted-foreground">
                <row.icon className="h-3.5 w-3.5 shrink-0" />
                <span className="text-[11px] tracking-[0.02em]">{row.label}</span>
              </div>
              <span className={`text-[11px] font-medium drop-shadow-[0_0_12px_rgba(255,255,255,0.04)] ${row.toneClass}`}>
                {row.value}
              </span>
            </div>
          ))}
        </div>

        {points.length === 0 ? (
          <p className="pointer-events-none absolute inset-x-0 bottom-6 text-center text-[7px] uppercase tracking-[0.18em] text-muted-foreground/35">
            Waiting for live ping data
          </p>
        ) : null}
      </div>

      <p className="mt-1 border-t border-white/6 pt-1 text-[7px] leading-[14px] tracking-[0.04em] text-muted-foreground/88">
        Service stops automatically if network or internet drops.
      </p>
    </div>
  );
}
