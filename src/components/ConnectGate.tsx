import { useEffect, useState } from "react";
import { Check, ExternalLink, Gamepad2, RefreshCw } from "lucide-react";
import { api } from "../lib/ipc";
import { useStore } from "../lib/store";
import { Button, cx } from "./primitives";

export function ConnectGate({ onSkip }: { onSkip: () => void }) {
  const refresh = useStore((s) => s.refresh);
  const [busy, setBusy] = useState(false);
  const [dots, setDots] = useState(0);
  const [page, setPage] = useState<"idle" | "opening" | "opened">("idle");

  const openPage = async () => {
    if (page !== "idle") return;
    setPage("opening");
    try {
      await api.openUrl("https://www.roblox.com/games/1730877806/Grand-Piece-Online");
      setPage("opened");
    } catch {
      setPage("idle");
      return;
    }
    setTimeout(() => setPage("idle"), 2500);
  };

  useEffect(() => {
    const t = setInterval(() => setDots((d) => (d + 1) % 4), 600);
    return () => clearInterval(t);
  }, []);

  const reload = async () => {
    setBusy(true);
    try {
      await refresh();
    } finally {
      setTimeout(() => setBusy(false), 400);
    }
  };

  return (
    <div className="h-full flex flex-col items-center justify-center text-center px-8 rise">
      <div className="relative mb-6">
        <div className="w-20 h-20 rounded-3xl bg-accent-soft grid place-items-center">
          <Gamepad2 size={34} className="text-accent" />
        </div>
        <span className="absolute -right-1 -bottom-1 w-5 h-5 rounded-full bg-bg-elev border border-line-strong grid place-items-center">
          <span className="w-2 h-2 rounded-full bg-warn pulse text-warn" />
        </span>
      </div>
      <div className="text-[17px] font-semibold mb-1.5">Connect to Grand Piece Online</div>
      <div className="text-fg-dim leading-relaxed max-w-[300px]">
        Open Roblox and join your GPO private server. The macro attaches to the game window automatically.
      </div>
      <div className="mt-2 text-[12px] text-fg-mute font-mono h-5">looking for Roblox{".".repeat(dots)}</div>
      <div className="mt-6 flex items-center gap-3">
        <Button kind="primary" onClick={reload} disabled={busy} icon={<RefreshCw size={14} className={cx(busy && "animate-spin")} />}>
          Reload
        </Button>
        <Button kind="ghost" onClick={openPage} disabled={page === "opening"} icon={page === "opened" ? <Check size={14} className="text-ok" /> : <ExternalLink size={14} />}>
          {page === "idle" ? "Open GPO page" : page === "opening" ? "Opening…" : "Opened"}
        </Button>
      </div>
      <button onClick={onSkip} className="mt-8 text-[11px] text-fg-mute hover:text-fg-dim underline-offset-2 hover:underline">
        Continue without Roblox
      </button>
    </div>
  );
}
