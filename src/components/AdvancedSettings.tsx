import { ChevronDown } from "lucide-react";
import { useState } from "react";
import type { KickInterval } from "@/hooks/useServiceState";

interface AdvancedSettingsProps {
  kickInterval: KickInterval;
  onKickIntervalChange: (value: KickInterval) => void;
}

const intervalOptions: { value: KickInterval; label: string }[] = [
  { value: "5", label: "5s" },
  { value: "10", label: "10s" },
  { value: "15", label: "15s" },
  { value: "20", label: "20s" },
  { value: "25", label: "25s" },
  { value: "30", label: "30s" },
];

export function AdvancedSettings({ kickInterval, onKickIntervalChange }: AdvancedSettingsProps) {
  const [open, setOpen] = useState(false);

  return (
    <div className="mx-3">
      <button
        onClick={() => setOpen(!open)}
        className="flex w-full items-center justify-between text-xs text-muted-foreground hover:text-foreground transition-colors py-1.5"
      >
        <span className="font-medium">Advanced</span>
        <ChevronDown className={`w-3.5 h-3.5 transition-transform duration-200 ${open ? "rotate-180" : ""}`} />
      </button>

      {open && (
        <div className="mt-1.5 space-y-2 rounded-lg glass-surface p-3 text-xs animate-in slide-in-from-top-1 duration-200">
          <div className="space-y-1">
            <span className="text-muted-foreground">Kick URL</span>
            <p className="text-foreground/80 font-mono text-[10px] truncate">
              https://selfcare.hutch.lk/selfcare/login.html
            </p>
          </div>

          <div className="flex items-center justify-between">
            <span className="text-muted-foreground">Kick Interval</span>
            <div className="flex max-w-[160px] flex-wrap justify-end gap-1">
              {intervalOptions.map((opt) => (
                <button
                  key={opt.value}
                  onClick={() => onKickIntervalChange(opt.value)}
                  className={`rounded-md px-2 py-0.5 text-[10px] font-medium transition-colors ${
                    kickInterval === opt.value
                      ? "bg-primary/20 text-primary"
                      : "text-muted-foreground hover:text-foreground hover:bg-secondary/50"
                  }`}
                >
                  {opt.label}
                </button>
              ))}
            </div>
          </div>

          <div className="space-y-1">
            <span className="text-muted-foreground">Internet Check</span>
            <p className="text-foreground/80 font-mono text-[10px]">
              HEAD https://www.gstatic.com/generate_204
            </p>
          </div>

          <p className="text-[10px] text-muted-foreground/70 italic">
            Default interval: 20s
          </p>
        </div>
      )}
    </div>
  );
}
