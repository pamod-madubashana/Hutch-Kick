import { Play, Square } from "lucide-react";
import type { KickInterval, ServiceStatus } from "@/hooks/useServiceState";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

interface PrimaryControlsProps {
  status: ServiceStatus;
  onStart: () => void;
  onStop: () => void;
  kickInterval: KickInterval;
  onKickIntervalChange: (value: KickInterval) => void;
  errorMessage: string | null;
}

const intervalOptions: { value: KickInterval; label: string }[] = [
  { value: "5", label: "5 seconds" },
  { value: "10", label: "10 seconds" },
  { value: "15", label: "15 seconds" },
  { value: "20", label: "20 seconds" },
  { value: "25", label: "25 seconds" },
  { value: "30", label: "30 seconds" },
];

export function PrimaryControls({
  status,
  onStart,
  onStop,
  kickInterval,
  onKickIntervalChange,
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

        <div className="w-[132px] shrink-0">
          <Select value={kickInterval} onValueChange={(value) => onKickIntervalChange(value as KickInterval)}>
            <SelectTrigger className="h-[42px] rounded-xl border-border/60 bg-secondary/20 px-3 text-xs font-semibold text-foreground shadow-none focus:ring-1 focus:ring-primary/40 focus:ring-offset-0">
              <SelectValue placeholder="Delay" />
            </SelectTrigger>
            <SelectContent className="rounded-xl border-border/50 bg-popover/95 backdrop-blur-xl">
              {intervalOptions.map((option) => (
                <SelectItem key={option.value} value={option.value} className="text-xs">
                  {option.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
      </div>
    </div>
  );
}
