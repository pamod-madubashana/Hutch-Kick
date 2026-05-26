import type { ServiceStatus } from "@/hooks/useServiceState";

const statusConfig: Record<ServiceStatus, { label: string; className: string }> = {
  RUNNING: { label: "RUNNING", className: "status-running" },
  STOPPED: { label: "STOPPED", className: "status-stopped" },
  STARTING: { label: "STARTING", className: "status-starting" },
  STOPPING: { label: "STOPPING", className: "status-stopping" },
  ERROR: { label: "ERROR", className: "status-error" },
};

interface PopoverHeaderProps {
  status: ServiceStatus;
}

export function PopoverHeader({ status }: PopoverHeaderProps) {
  const config = statusConfig[status];

  return (
    <div className="flex items-start justify-between gap-3 px-4 py-3.5">
      <div>
        <h1 className="text-sm font-semibold tracking-tight text-foreground">Hutch-Kick</h1>
        <p className="mt-1 text-[10px] uppercase tracking-[0.16em] text-muted-foreground/75">
          Connectivity watchdog
        </p>
      </div>
      <div className="flex items-center gap-2 rounded-full border border-border/50 bg-secondary/35 px-2.5 py-1 shadow-[inset_0_1px_0_rgba(255,255,255,0.04)]">
        <span className={`status-dot ${config.className}`} />
        <span className="text-[10px] font-medium tracking-wider text-muted-foreground uppercase">
          {config.label}
        </span>
      </div>
    </div>
  );
}
