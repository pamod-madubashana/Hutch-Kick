import { TimerReset } from "lucide-react";
import type { KickInterval } from "@/hooks/useServiceState";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

interface DelaySelectorProps {
  kickInterval: KickInterval;
  onKickIntervalChange: (value: KickInterval) => void;
}

const intervalOptions: { value: KickInterval; label: string }[] = [
  { value: "5", label: "5 seconds" },
  { value: "10", label: "10 seconds" },
  { value: "15", label: "15 seconds" },
  { value: "20", label: "20 seconds" },
  { value: "25", label: "25 seconds" },
  { value: "30", label: "30 seconds" },
];

export function DelaySelector({ kickInterval, onKickIntervalChange }: DelaySelectorProps) {
  return (
    <div className="mx-3 rounded-xl border border-border/50 bg-card/70 p-3 shadow-[0_16px_40px_-30px_rgba(0,0,0,0.8)] backdrop-blur-xl">
      <div className="flex items-start justify-between gap-3">
        <div className="flex items-start gap-2.5">
          <div className="mt-0.5 rounded-lg border border-primary/20 bg-primary/10 p-1.5 text-primary">
            <TimerReset className="h-3.5 w-3.5" />
          </div>
          <div>
            <p className="text-[10px] font-semibold uppercase tracking-[0.18em] text-muted-foreground/80">Delay</p>
            <p className="mt-1 text-xs font-medium text-foreground">Auto kick interval</p>
            <p className="text-[10px] text-muted-foreground">Choose how often the service refreshes the connection.</p>
          </div>
        </div>

        <div className="min-w-[118px] shrink-0">
          <Select value={kickInterval} onValueChange={(value) => onKickIntervalChange(value as KickInterval)}>
            <SelectTrigger className="h-9 rounded-lg border-border/50 bg-secondary/30 text-xs font-medium shadow-none focus:ring-1 focus:ring-primary/40 focus:ring-offset-0">
              <SelectValue />
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
