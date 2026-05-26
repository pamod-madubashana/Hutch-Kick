import { Copy } from "lucide-react";
import type { LogEntry } from "@/hooks/useServiceState";

interface LogPanelProps {
  logs: LogEntry[];
}

function getLatencyColorClass(latencyMs: number): string {
  if (latencyMs < 70) return "text-status-good";
  if (latencyMs <= 100) return "text-status-warn";
  if (latencyMs <= 200) return "text-status-alert";
  return "text-status-bad";
}

function getStatusColorClass(token: string): string | null {
  switch (token.toLowerCase()) {
    case "connected":
    case "online":
      return "text-status-good";
    case "unknown":
      return "text-status-warn";
    case "disconnected":
    case "offline":
      return "text-status-bad";
    default:
      return null;
  }
}

function renderStatusTokens(text: string, keyPrefix: string) {
  return text.split(/\b(connected|disconnected|unknown|online|offline)\b/gi).map((part, index) => {
    const colorClass = getStatusColorClass(part);

    if (!colorClass) {
      return <span key={`${keyPrefix}-${index}`}>{part}</span>;
    }

    return (
      <span key={`${keyPrefix}-${index}`} className={`font-medium ${colorClass}`}>
        {part}
      </span>
    );
  });
}

function renderLogMessage(message: string) {
  const latencyMatch = message.match(/\b(\d+(?:\.\d+)?)ms\b/i);

  if (!latencyMatch || latencyMatch.index === undefined) {
    return renderStatusTokens(message, "message");
  }

  const latencyValue = Number(latencyMatch[1]);
  const latencyText = latencyMatch[0];
  const before = message.slice(0, latencyMatch.index);
  const after = message.slice(latencyMatch.index + latencyText.length);

  return (
    <>
      {renderStatusTokens(before, "before")}
      <span className={`font-medium ${getLatencyColorClass(latencyValue)}`}>{latencyText}</span>
      {renderStatusTokens(after, "after")}
    </>
  );
}

function formatTime(date: Date): string {
  return date.toLocaleTimeString("en-US", { hour12: false, hour: "2-digit", minute: "2-digit", second: "2-digit" });
}

export function LogPanel({ logs }: LogPanelProps) {
  const handleCopy = () => {
    const text = logs.map((l) => `[${formatTime(l.timestamp)}] ${l.message}`).join("\n");
    navigator.clipboard.writeText(text);
  };

  return (
    <div className="mx-3 flex-1 flex flex-col min-h-0">
      <div className="mb-1.5 flex items-center justify-between">
        <span className="text-[10px] font-medium text-muted-foreground uppercase tracking-wider">
          Recent Events
        </span>
        <div className="flex gap-1">
          <button
            onClick={handleCopy}
            className="rounded p-1 text-muted-foreground hover:text-foreground hover:bg-secondary/50 transition-colors"
            title="Copy logs"
          >
            <Copy className="w-3 h-3" />
          </button>
        </div>
      </div>

      <div className="themed-scrollbar flex-1 min-h-[60px] space-y-0.5 overflow-y-auto rounded-xl border border-border/30 bg-secondary/16 p-2 shadow-[inset_0_1px_0_rgba(255,255,255,0.03)]">
        {logs.length === 0 ? (
          <p className="text-[10px] text-muted-foreground/50 text-center py-2">No events</p>
        ) : (
          logs.map((entry) => (
            <div key={entry.id} className="flex gap-2 rounded-md px-1.5 py-1 text-[10px] leading-relaxed hover:bg-secondary/20">
              <span className="text-muted-foreground/60 font-mono shrink-0">
                {formatTime(entry.timestamp)}
              </span>
              <span className="text-foreground/80">{renderLogMessage(entry.message)}</span>
            </div>
          ))
        )}
      </div>
    </div>
  );
}
