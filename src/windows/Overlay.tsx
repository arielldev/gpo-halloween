import { useEffect, useRef, useState } from "react";
import { api, on } from "../lib/ipc";
import type { OverlaySession, RelPoint, RelRect, TimerTest } from "../lib/types";
import { fmtClock } from "../lib/types";
import { cx } from "../components/primitives";

type PxBox = { x: number; y: number; w: number; h: number };
type Drag = { type: "move" | "draw" | "resize"; edge?: string; sx: number; sy: number; start: RelRect };

const HANDLE = 8;
const ACCENT = "#ff7a1a";

export default function Overlay() {
  const [session, setSession] = useState<OverlaySession | null>(null);
  const [region, setRegion] = useState<RelRect | null>(null);
  const [point, setPoint] = useState<RelPoint | null>(null);
  const [size, setSize] = useState({ w: window.innerWidth, h: window.innerHeight });
  const drag = useRef<Drag | null>(null);
  const [live, setLive] = useState<TimerTest | null>(null);
  const regionRef = useRef<RelRect | null>(null);
  regionRef.current = region;

  useEffect(() => {
    const open = (s: OverlaySession) => {
      setRegion(s.region);
      setPoint(s.point);
      setSession(s);
    };
    const sync = async () => {
      try {
        const p = await api.overlayPending();
        if (p) open(p);
        else setSession(null);
      } catch {
        /* backend not ready */
      }
    };
    const subs = [on("overlay:session", open), on("overlay:close", () => setSession(null))];
    const onResize = () => setSize({ w: window.innerWidth, h: window.innerHeight });
    const onVisible = () => {
      if (document.visibilityState === "visible") sync();
    };
    window.addEventListener("resize", onResize);
    window.addEventListener("focus", sync);
    document.addEventListener("visibilitychange", onVisible);
    sync();
    return () => {
      subs.forEach((p) => p.then((u) => u()));
      window.removeEventListener("resize", onResize);
      window.removeEventListener("focus", sync);
      document.removeEventListener("visibilitychange", onVisible);
    };
  }, []);

  const isRegion = session?.target.kind === "timer_region";

  useEffect(() => {
    if (!isRegion) return;
    let alive = true;
    const tick = async () => {
      const r = regionRef.current;
      if (!r || r.w < 0.005 || r.h < 0.005 || drag.current) return;
      try {
        const t = await api.timerTest(r);
        if (alive) setLive(t);
      } catch {
        if (alive) setLive(null);
      }
    };
    tick();
    const t = setInterval(tick, 900);
    return () => {
      alive = false;
      clearInterval(t);
      setLive(null);
    };
  }, [isRegion, session]);
  const canSave = isRegion ? !!region && region.w >= 0.005 && region.h >= 0.005 : !!point;

  const cancel = () => {
    setSession(null);
    api.overlayCancel();
  };

  const commit = async () => {
    if (!session || !canSave) return;
    await api.overlayCommit({ target: session.target, region: isRegion ? region : null, point: isRegion ? null : point });
    setSession(null);
  };

  useEffect(() => {
    const h = (e: KeyboardEvent) => {
      if (!session) return;
      if (e.key === "Escape") cancel();
      else if (e.key === "Enter") commit();
      else if (isRegion && region && ["ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight"].includes(e.key)) {
        e.preventDefault();
        const step = (e.shiftKey ? 10 : 1) / (e.key === "ArrowUp" || e.key === "ArrowDown" ? size.h : size.w);
        const r = { ...region };
        if (e.key === "ArrowUp") r.y -= step;
        if (e.key === "ArrowDown") r.y += step;
        if (e.key === "ArrowLeft") r.x -= step;
        if (e.key === "ArrowRight") r.x += step;
        setRegion(r);
      }
    };
    window.addEventListener("keydown", h);
    return () => window.removeEventListener("keydown", h);
  });

  if (!session) return null;

  const { w: W, h: H } = size;
  const toPx = (r: RelRect): PxBox => ({ x: r.x * W, y: r.y * H, w: r.w * W, h: r.h * H });
  const px = region ? toPx(region) : null;
  const index = session.target.kind === "step_point" ? session.target.index : -1;

  const onDown = (e: React.PointerEvent) => {
    if (e.button !== 0) return;
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    const p = { x: clamp01(e.clientX / W), y: clamp01(e.clientY / H) };
    if (!isRegion) {
      setPoint(p);
      return;
    }
    const edge = px ? hitEdge(px, e.clientX, e.clientY) : null;
    if (region && edge === "inside") {
      drag.current = { type: "move", sx: e.clientX, sy: e.clientY, start: region };
    } else if (region && edge) {
      drag.current = { type: "resize", edge, sx: e.clientX, sy: e.clientY, start: region };
    } else {
      const start = { x: p.x, y: p.y, w: 0, h: 0 };
      setRegion(start);
      drag.current = { type: "draw", sx: e.clientX, sy: e.clientY, start };
    }
  };

  const onMove = (e: React.PointerEvent) => {
    const d = drag.current;
    if (!d) return;
    setRegion(applyDrag(d, e.clientX, e.clientY, W, H));
  };

  const cursorFor = (e: React.MouseEvent) => {
    if (!isRegion || !px) return "crosshair";
    const edge = hitEdge(px, e.clientX, e.clientY);
    return edge === "inside" ? "move" : edge ? `${edge}-resize` : "crosshair";
  };

  const path = session.path.map((p, i) => (i === index ? point : p));

  return (
    <div
      className="w-full h-full relative select-none"
      onPointerDown={onDown}
      onPointerMove={onMove}
      onPointerUp={() => (drag.current = null)}
      onMouseMove={(e) => ((e.currentTarget as HTMLElement).style.cursor = cursorFor(e))}
      style={{ background: isRegion ? "rgba(8,5,12,0.12)" : "rgba(8,5,12,0.40)" }}
    >
      {isRegion && px && (
        <>
          <div className="absolute inset-0 pointer-events-none" style={maskStyle(px, W, H)} />
          <Box
            r={px}
            color={ACCENT}
            label="1 · Server timer"
            badge={
              <span className="text-[10px] font-mono px-1.5 rounded-sm whitespace-nowrap" style={{ background: live?.seconds != null ? "#22c55e" : "#f59e0b", color: "#000" }}>
                {live ? (live.seconds != null ? `reads ${fmtClock(live.seconds)}` : "no time found") : "reading…"}
              </span>
            }
          />
        </>
      )}
      {!isRegion && <Route path={path} active={index} W={W} H={H} />}
      <Toolbar
        title={session.title}
        hint={
          isRegion
            ? "Box the server timer (bottom right) · drag to move · edges resize · arrows nudge (Shift ×10) · Enter save · Esc cancel"
            : "Click where this step should click · numbered dots are the other clicks in this sequence · Enter save · Esc cancel"
        }
        onSave={commit}
        onCancel={cancel}
        canSave={canSave}
      />
    </div>
  );
}

function Route({ path, active, W, H }: { path: Array<RelPoint | null>; active: number; W: number; H: number }) {
  const pts = path.map((p, i) => (p ? { i, x: p.x * W, y: p.y * H } : null)).filter(Boolean) as { i: number; x: number; y: number }[];
  return (
    <>
      <svg className="absolute inset-0 pointer-events-none" width={W} height={H}>
        {pts.slice(1).map((p, k) => (
          <line key={k} x1={pts[k].x} y1={pts[k].y} x2={p.x} y2={p.y} stroke="rgba(255,255,255,0.25)" strokeWidth={1.5} strokeDasharray="4 4" />
        ))}
      </svg>
      {pts.map((p) =>
        p.i === active ? (
          <Cross key={p.i} x={p.x} y={p.y} n={p.i + 1} />
        ) : (
          <div
            key={p.i}
            className="absolute pointer-events-none -translate-x-1/2 -translate-y-1/2 w-5 h-5 rounded-full grid place-items-center text-[10px] font-mono font-semibold bg-black/70 text-fg-dim border border-white/25"
            style={{ left: p.x, top: p.y }}
          >
            {p.i + 1}
          </div>
        ),
      )}
    </>
  );
}

function applyDrag(d: Drag, cx0: number, cy0: number, W: number, H: number): RelRect {
  const dx = (cx0 - d.sx) / W;
  const dy = (cy0 - d.sy) / H;
  if (d.type === "move") {
    return { ...d.start, x: clamp01(d.start.x + dx, 1 - d.start.w), y: clamp01(d.start.y + dy, 1 - d.start.h) };
  }
  if (d.type === "draw") {
    return {
      x: clamp01(Math.min(d.start.x, d.start.x + dx)),
      y: clamp01(Math.min(d.start.y, d.start.y + dy)),
      w: Math.abs(dx),
      h: Math.abs(dy),
    };
  }
  let { x, y, w, h } = d.start;
  const edge = d.edge ?? "";
  if (edge.includes("w")) {
    x += dx;
    w -= dx;
  }
  if (edge.includes("e")) w += dx;
  if (edge.includes("n")) {
    y += dy;
    h -= dy;
  }
  if (edge.includes("s")) h += dy;
  return { x, y, w: Math.max(0.005, w), h: Math.max(0.005, h) };
}

function clamp01(v: number, max = 1) {
  return Math.max(0, Math.min(max, v));
}

function hitEdge(r: PxBox, x: number, y: number): string | null {
  const inX = x >= r.x - HANDLE && x <= r.x + r.w + HANDLE;
  const inY = y >= r.y - HANDLE && y <= r.y + r.h + HANDLE;
  if (!inX || !inY) return null;
  const v = Math.abs(y - r.y) <= HANDLE ? "n" : Math.abs(y - (r.y + r.h)) <= HANDLE ? "s" : "";
  const h = Math.abs(x - r.x) <= HANDLE ? "w" : Math.abs(x - (r.x + r.w)) <= HANDLE ? "e" : "";
  return v || h ? `${v}${h}` : "inside";
}

function maskStyle(r: PxBox, W: number, H: number): React.CSSProperties {
  const path = `polygon(0 0, ${W}px 0, ${W}px ${H}px, 0 ${H}px, 0 0, ${r.x}px ${r.y}px, ${r.x}px ${r.y + r.h}px, ${r.x + r.w}px ${r.y + r.h}px, ${r.x + r.w}px ${r.y}px, ${r.x}px ${r.y}px)`;
  return { background: "rgba(0,0,0,0.38)", clipPath: path };
}

function Box({ r, color, label, badge }: { r: PxBox; color: string; label: string; badge?: React.ReactNode }) {
  return (
    <div
      className="absolute pointer-events-none"
      style={{ left: r.x, top: r.y, width: r.w, height: r.h, boxShadow: `0 0 0 2px ${color}, 0 0 22px ${color}66` }}
    >
      <div className="absolute -top-5 left-0 flex gap-1 items-center">
        <span className="text-[10px] font-mono px-1.5 rounded-sm whitespace-nowrap" style={{ background: color, color: "#000" }}>
          {label}
        </span>
        {badge}
      </div>
      {["nw", "ne", "sw", "se"].map((k) => (
        <div
          key={k}
          className="absolute w-2.5 h-2.5 rounded-sm bg-white"
          style={{
            left: k.includes("w") ? -5 : undefined,
            right: k.includes("e") ? -5 : undefined,
            top: k.includes("n") ? -5 : undefined,
            bottom: k.includes("s") ? -5 : undefined,
            boxShadow: `0 0 0 1.5px ${color}`,
          }}
        />
      ))}
    </div>
  );
}

function Cross({ x, y, n }: { x: number; y: number; n: number }) {
  return (
    <div className="absolute pointer-events-none" style={{ left: x, top: y }}>
      <div className="absolute -left-4 -top-px w-8 h-0.5 bg-accent" />
      <div className="absolute -top-4 -left-px h-8 w-0.5 bg-accent" />
      <div className="absolute -left-3 -top-3 w-6 h-6 rounded-full border-2 border-accent" />
      <div className="absolute left-4 -top-6 text-[10px] font-mono font-semibold px-1.5 rounded-sm bg-accent text-black">{n}</div>
    </div>
  );
}

function Toolbar({ title, hint, onSave, onCancel, canSave }: { title: string; hint: string; onSave: () => void; onCancel: () => void; canSave: boolean }) {
  return (
    <div
      className="absolute left-1/2 -translate-x-1/2 bottom-6 glass rounded-2xl px-4 py-3 flex flex-wrap items-center justify-center gap-x-4 gap-y-2 shadow-[0_10px_40px_rgba(0,0,0,0.5)] max-w-[min(92vw,900px)]"
      onPointerDown={(e) => e.stopPropagation()}
    >
      <div className="min-w-0 basis-full sm:basis-auto sm:flex-1 text-center sm:text-left">
        <div className="font-semibold text-[13px]">{title}</div>
        <div className="text-[11px] text-fg-dim leading-snug">{hint}</div>
      </div>
      <div className="flex gap-2 shrink-0">
        <button onClick={onCancel} className="h-8 px-3 rounded-lg bg-white/[0.06] hover:bg-white/[0.12] text-[12px] whitespace-nowrap">
          Cancel <kbd className="font-mono text-[10px] text-fg-mute ml-1">Esc</kbd>
        </button>
        <button
          onClick={onSave}
          disabled={!canSave}
          className={cx("h-8 px-3 rounded-lg text-[12px] font-medium whitespace-nowrap", canSave ? "bg-accent text-white hover:brightness-110" : "bg-white/[0.08] text-fg-mute")}
        >
          Save <kbd className="font-mono text-[10px] opacity-70 ml-1">Enter</kbd>
        </button>
      </div>
    </div>
  );
}
