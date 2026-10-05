import { useEffect, useRef, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Hand, PanelRightClose, PanelRightOpen, Play, RotateCw, Square, Timer } from "lucide-react";
import { api, on } from "../lib/ipc";
import { useStore, isActive } from "../lib/store";
import { SEQ_LABEL, STATE_LABEL, fmtClock } from "../lib/types";
import { cx, fmtRuntime } from "../components/primitives";
import { StateBadge } from "../components/StateIcon";

export default function Hud() {
  const init = useStore((s) => s.init);
  const state = useStore((s) => s.state);
  const stats = useStore((s) => s.stats);
  const roblox = useStore((s) => s.roblox);
  const timer = useStore((s) => s.timer);
  const recording = useStore((s) => s.recording);
  const progress = useStore((s) => s.progress);
  const lastEvent = useStore((s) => s.lastEvent);
  const settings = useStore((s) => s.settings);
  const [runtime, setRuntime] = useState(stats.runtime_s);
  const [panelOpen, setPanelOpen] = useState(false);
  const dragging = useRef(false);

  useEffect(() => {
    init();
    api.panelVisible().then(setPanelOpen).catch(() => undefined);
    const un = on("panel:visible", setPanelOpen);
    return () => {
      un.then((u) => u());
    };
  }, [init]);

  const active = isActive(state);
  const farming = active && state !== "recording" && state !== "test_run";

  useEffect(() => {
    setRuntime(stats.runtime_s);
    if (!farming) return;
    const t = setInterval(() => setRuntime((r) => r + 1), 1000);
    return () => clearInterval(t);
  }, [stats.runtime_s, farming]);

  const dim = roblox && !roblox.is_foreground;
  const flash = lastEvent && Date.now() - lastEvent.ts < 6000 ? lastEvent : null;
  const rec = state === "recording";
  const limit = settings?.timer.stop_at_s ?? 290;
  const near = (secs: number) => (settings?.timer.mode === "remaining" ? secs <= limit + 30 : secs >= limit - 30);

  const title = rec
    ? `Recording ${SEQ_LABEL[recording?.seq ?? "macro"]} · ${recording?.steps ?? 0}`
    : flash
      ? flash.text
      : progress && active && progress.total > 0
        ? `${STATE_LABEL[state]} · ${Math.min(progress.index + 1, progress.total)}/${progress.total}`
        : STATE_LABEL[state];

  const startDrag = async (e: React.PointerEvent) => {
    if (e.button !== 0) return;
    dragging.current = true;
    const win = getCurrentWindow();
    await win.startDragging();
    dragging.current = false;
    const pos = await win.outerPosition();
    const size = await win.outerSize();
    const rb = useStore.getState().roblox;
    if (!rb) return;
    const cx0 = pos.x + size.width / 2 - rb.client.x;
    const cy0 = pos.y - rb.client.y - 8;
    api.hudSetOffset({ x: Math.max(0, Math.min(1, cx0 / rb.client.w)), y: Math.max(0, Math.min(1, cy0 / rb.client.h)) });
  };

  return (
    <div className={cx("h-full w-full flex items-center px-1.5 transition-opacity duration-300", dim && "opacity-85 hover:opacity-100")} onPointerDown={startDrag}>
      <div className={cx("glass rounded-full h-[44px] w-full flex items-center gap-2 pl-3 pr-1.5", rec && "border-rec/60!")}>
        <StateBadge state={state} size={28} icon={14} className="-ml-1" />
        <div className="min-w-0 flex-1 leading-tight">
          <div className="text-[12px] font-semibold truncate">{title}</div>
          <div className="text-[10.5px] text-fg-dim font-mono tabular-nums flex gap-2">
            <span className="inline-flex items-center gap-0.5">
              <Hand size={10} />
              {stats.houses}
            </span>
            <span className="inline-flex items-center gap-0.5">
              <RotateCw size={10} />
              {stats.cycles}
            </span>
            <span
              className={cx(
                "inline-flex items-center gap-0.5",
                timer?.armed && timer.seconds != null && (timer.source === "clock" || near(timer.seconds) ? "text-warn" : "text-ok"),
              )}
            >
              <Timer size={10} />
              {timer?.seconds != null ? fmtClock(timer.seconds) : "--:--"}
            </span>
            <span>{fmtRuntime(runtime)}</span>
          </div>
        </div>
        <button
          className={cx(
            "no-drag h-8 w-8 rounded-full grid place-items-center transition-colors",
            active ? "bg-bad-soft text-bad hover:bg-bad/30" : "bg-ok-soft text-ok hover:bg-ok/30",
          )}
          onPointerDown={(e) => e.stopPropagation()}
          onClick={() => (active ? api.botStop() : api.botToggle())}
          title={active ? "Stop" : "Start"}
        >
          {active ? <Square size={12} /> : <Play size={14} className="translate-x-px" />}
        </button>
        <button
          className={cx(
            "no-drag h-8 w-8 rounded-full grid place-items-center transition-colors",
            panelOpen ? "bg-accent-soft text-accent hover:bg-accent/30" : "bg-white/[0.06] text-fg-dim hover:bg-white/[0.12] hover:text-fg",
          )}
          onPointerDown={(e) => e.stopPropagation()}
          onClick={() => api.panelToggle()}
          title={panelOpen ? "Hide panel" : "Open panel"}
        >
          {panelOpen ? <PanelRightClose size={14} /> : <PanelRightOpen size={14} />}
        </button>
      </div>
    </div>
  );
}
