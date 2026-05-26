import { Play, Square, Zap } from "lucide-react";
import type { ServiceStatus } from "@/hooks/useServiceState";

interface PrimaryControlsProps {
  status: ServiceStatus;
  onStart: () => void;
  onStop: () => void;
  onKickNow: () => void;
  errorMessage: string | null;
}

export function PrimaryControls({
  status,
  onStart,
  onStop,
  onKickNow,
  errorMessage,
}: PrimaryControlsProps) {
  const isRunning = status === "RUNNING";
  const isStarting = status === "STARTING";
  const isStopping = status === "STOPPING";

  return (
    <div className="mx-3 space-y-2.5">
      {errorMessage && (
        <div className="rounded-xl border border-destructive/20 bg-destructive/10 px-3 py-2 text-xs text-destructive shadow-[0_12px_32px_-28px_rgba(239,68,68,0.95)]">
          <span>{errorMessage}</span>
        </div>
      )}

      <div className="flex gap-2">
        <button
          onClick={isRunning ? onStop : onStart}
          disabled={isStarting || isStopping}
          className={`flex flex-1 items-center justify-center gap-2 rounded-xl px-3 py-2.5 text-xs font-semibold transition-all duration-200 disabled:opacity-50 ${
             isRunning
              ? "border border-destructive/25 bg-destructive/10 text-destructive hover:bg-destructive/18 shadow-[0_16px_36px_-28px_rgba(239,68,68,0.95)]"
              : "bg-primary text-primary-foreground hover:opacity-95 shadow-[0_18px_38px_-24px_rgba(34,197,94,0.95)]"
           }`}
        >
          {isRunning ? (
            <>
              <Square className="w-3.5 h-3.5" /> Stop Service
            </>
          ) : isStarting ? (
            <>
              <div className="w-3.5 h-3.5 border-2 border-primary-foreground/30 border-t-primary-foreground rounded-full animate-spin" />
              Starting…
            </>
          ) : isStopping ? (
            <>
              <div className="w-3.5 h-3.5 border-2 border-primary-foreground/30 border-t-primary-foreground rounded-full animate-spin" />
              Stopping…
            </>
          ) : (
            <>
              <Play className="w-3.5 h-3.5" /> Start Service
            </>
          )}
        </button>

        <button
          onClick={onKickNow}
          disabled={!isRunning}
          className="flex items-center gap-1.5 rounded-xl border border-border/60 bg-secondary/20 px-3 py-2.5 text-xs font-semibold text-foreground transition-all duration-200 hover:bg-accent/70 disabled:cursor-not-allowed disabled:opacity-30"
        >
          <Zap className="w-3.5 h-3.5" /> Kick Now
        </button>
      </div>
    </div>
  );
}
