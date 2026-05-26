interface PopoverFooterProps {
  version: string;
  onQuit: () => void;
}

export function PopoverFooter({ version, onQuit }: PopoverFooterProps) {
  return (
    <div className="flex items-center justify-between border-t border-border/30 px-4 py-2.5">
      <div>
        <span className="font-mono text-[10px] text-muted-foreground/60">{version}</span>
        <p className="text-[9px] uppercase tracking-[0.14em] text-muted-foreground/45">Tray ready</p>
      </div>
      <div className="flex items-center gap-3">
        <button
          onClick={onQuit}
          className="rounded-full border border-border/50 px-2.5 py-1 text-[10px] font-medium text-muted-foreground transition-colors hover:border-destructive/30 hover:text-destructive"
        >
          Quit
        </button>
      </div>
    </div>
  );
}
