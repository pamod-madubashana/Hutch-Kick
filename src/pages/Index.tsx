import { useServiceState } from "@/hooks/useServiceState";
import { PopoverHeader } from "@/components/PopoverHeader";
import { StatusCard } from "@/components/StatusCard";
import { PrimaryControls } from "@/components/PrimaryControls";
import { LogPanel } from "@/components/LogPanel";
import { PopoverFooter } from "@/components/PopoverFooter";

const Index = () => {
  const service = useServiceState();

  return (
    <div className="relative h-screen w-screen overflow-hidden bg-background">
      <div className="pointer-events-none absolute inset-0 bg-[radial-gradient(circle_at_top,rgba(70,196,123,0.18),transparent_46%),linear-gradient(180deg,rgba(255,255,255,0.02),transparent_28%)]" />
      <div className="pointer-events-none absolute -left-14 top-24 h-32 w-32 rounded-full bg-primary/10 blur-3xl" />
      <div className="pointer-events-none absolute right-0 top-0 h-24 w-24 rounded-full bg-white/5 blur-2xl" />

      <div className="relative flex h-full w-full flex-col overflow-hidden border border-border/40 bg-card/88 shadow-[0_24px_80px_-42px_rgba(0,0,0,0.95)] backdrop-blur-2xl">
        <PopoverHeader status={service.status} />

        <div className="h-px bg-border/30 mx-3" />

        <div className="flex-1 flex flex-col gap-2.5 py-3 overflow-hidden">
          <StatusCard
            wifiStatus={service.wifiStatus}
            internetStatus={service.internetStatus}
            lastKick={service.lastKick}
            logs={service.logs}
          />

          <PrimaryControls
            status={service.status}
            onStart={service.startService}
            onStop={service.stopService}
            kickInterval={service.kickInterval}
            onKickIntervalChange={service.setKickInterval}
            errorMessage={service.errorMessage}
          />

          {!service.backendConnected && (
            <p className="mx-3 rounded-md border border-destructive/20 bg-destructive/10 px-3 py-2 text-[10px] text-destructive">
              Backend unavailable. Run this UI with Tauri desktop runtime.
            </p>
          )}

          <LogPanel logs={service.logs} />
        </div>

        <PopoverFooter version="v0.1.1" onQuit={service.quitApp} />
      </div>
    </div>
  );
};

export default Index;
