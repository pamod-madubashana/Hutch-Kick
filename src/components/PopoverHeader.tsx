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
    <div className="flex items-center justify-between px-4 py-3">
      <div>
        <h1 className="text-sm font-semibold text-foreground tracking-tight">Hutch-Kick</h1>
      </div>
      <div className="flex items-center gap-2 rounded-full bg-secondary/50 px-2.5 py-1">
        <span className={`status-dot ${config.className}`} />
        <span className="text-[10px] font-medium tracking-wider text-muted-foreground uppercase">
          {config.label}
        </span>
      </div>
    </div>
  );
}
