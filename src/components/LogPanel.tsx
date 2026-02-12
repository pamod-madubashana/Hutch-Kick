import { Copy } from "lucide-react";
import type { LogEntry } from "@/hooks/useServiceState";

interface LogPanelProps {
  logs: LogEntry[];
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
      <div className="flex items-center justify-between mb-1.5">
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

      <div className="flex-1 overflow-hidden rounded-lg bg-secondary/20 border border-border/30 p-2 space-y-0.5 min-h-[60px]">
        {logs.length === 0 ? (
          <p className="text-[10px] text-muted-foreground/50 text-center py-2">No events</p>
        ) : (
          logs.map((entry) => (
            <div key={entry.id} className="flex gap-2 text-[10px] leading-relaxed">
              <span className="text-muted-foreground/60 font-mono shrink-0">
                {formatTime(entry.timestamp)}
              </span>
              <span className="text-foreground/80">{entry.message}</span>
            </div>
          ))
        )}
      </div>
    </div>
  );
}
