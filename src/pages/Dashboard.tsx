import { useEffect, useRef, useState } from "react";
import { BookOpen, ChevronDown, Play, Square } from "lucide-react";
import { api } from "../lib/ipc";
import { useStore, isActive } from "../lib/store";
import { SEQ_LABEL, STATE_LABEL, activeOrder, fmtClock } from "../lib/types";
import { Button, Pill, Section, TextField, cx, fmtRuntime } from "../components/primitives";
import { StateBadge } from "../components/StateIcon";
import { LogList } from "../components/LogList";
import { GetStarted, setupStatus } from "../components/GetStarted";
import { RunOrderSummary } from "../components/RunOrder";
import { DiscordCard } from "../components/Discord";

export default function Dashboard() {
  const state = useStore((s) => s.state);
  const detail = useStore((s) => s.detail);
  const stats = useStore((s) => s.stats);
  const roblox = useStore((s) => s.roblox);
  const settings = useStore((s) => s.settings);
  const progress = useStore((s) => s.progress);
  const [runtime, setRuntime] = useState(stats.runtime_s);
  const active = isActive(state);
  const farming = active && state !== "recording" && state !== "test_run";
  const total = stats.total;
  const totalRuntime = total.runtime_s + (runtime - stats.runtime_s);

  useEffect(() => {
    setRuntime(stats.runtime_s);
    if (!farming) return;
    const t = setInterval(() => setRuntime((r) => r + 1), 1000);
    return () => clearInterval(t);
  }, [stats.runtime_s, farming]);

  if (!settings) return null;
  const sub = (n: number) => (farming ? `this run ${n}` : undefined);
  const step = progress && active && progress.total > 0 ? `${SEQ_LABEL[progress.seq]} step ${Math.min(progress.index + 1, progress.total)} of ${progress.total}` : null;

  return (
    <div className="pb-4">
      <div className="px-4 pt-4 pb-3 flex items-center gap-3">
        <StateBadge state={state} />
        <div className="flex-1 min-w-0">
          <div className="text-[15px] font-semibold leading-tight">{STATE_LABEL[state]}</div>
          <div className="text-[12px] text-fg-dim truncate">
            {step ?? detail ?? (roblox ? (roblox.is_foreground ? "Roblox focused" : "Roblox in background") : "Roblox not detected")}
          </div>
        </div>
        {active ? (
          <Button kind="danger" onClick={() => api.botStop()} icon={<Square size={13} />}>
            Stop
          </Button>
        ) : (
          <Button kind="primary" onClick={() => api.botStart()} icon={<Play size={14} />}>
            Start
          </Button>
        )}
      </div>
      <div className="px-4 pb-3">
        <ServerCodeCard />
      </div>
      <div className="px-4 pb-3">
        <StartHint />
      </div>
      <div className="px-4 pb-4">
        <RunOrderSummary />
      </div>

      <GetStartedSection />

      <div className="px-4 pb-6">
        <DiscordCard />
      </div>

      <Section title="Server timer">
        <TimerCard />
      </Section>

      <Section title="All time">
        <Stat label="Houses visited" value={total.houses} sub={sub(stats.houses)} />
        <Stat label="Routes finished" value={total.laps} sub={sub(stats.laps)} />
        {total.buys > 0 && <Stat label="Shop trips" value={total.buys} sub={sub(stats.buys)} />}
        <Stat label="Rejoins" value={total.cycles} sub={sub(stats.cycles)} />
        <Stat label="Timer stops" value={total.timer_trips} sub={sub(stats.timer_trips)} />
        <Stat label="Time farming" value={fmtRuntime(totalRuntime)} sub={farming ? `this run ${fmtRuntime(runtime)}` : undefined} />
        <Stat label="Sessions" value={total.sessions} />
        {stats.restarts > 0 && <Stat label="Recoveries" value={stats.restarts} />}
      </Section>

      <Section title="Activity">
        <LogList height={220} />
      </Section>
    </div>
  );
}

function ServerCodeCard() {
  const settings = useStore((s) => s.settings);
  const update = useStore((s) => s.update);
  if (!settings) return null;
  const code = settings.server.code.trim();
  return (
    <div className={cx("rounded-xl border px-3 py-2.5", code ? "border-line bg-black/20" : "border-bad/50 bg-bad-soft")}>
      <div className="flex items-center gap-2 mb-1.5">
        <span className="text-[12.5px] font-semibold">Private server code</span>
        <Pill tone={code ? "ok" : "bad"}>{code ? "set" : "required"}</Pill>
      </div>
      <TextField
        value={settings.server.code}
        onChange={(v) => update((x) => void (x.server.code = v.trim()))}
        placeholder="paste your private server code here"
        mono
      />
      <div className="mt-1.5 text-[11px] text-fg-mute leading-snug">Every rejoin pastes this code into GPO's private server box.</div>
    </div>
  );
}

function StartHint() {
  const settings = useStore((s) => s.settings);
  if (!settings) return null;
  const first = activeOrder(settings)[0];
  const where =
    first === "lobby" ? (
      <>
        Open GPO and stay on the <b className="text-fg">main menu (lobby)</b>, then press
      </>
    ) : first === "leave" ? (
      <>
        Join your private server, then press
      </>
    ) : (
      <>
        Stand at your spawn in your private server, then press
      </>
    );
  return (
    <div className="rounded-xl border border-accent/30 bg-accent-soft px-3 py-2.5 text-[12px] text-fg-dim leading-relaxed">
      {where} <span className="font-mono text-fg">{settings.hotkeys.toggle}</span> (or Start). Press it again any time to stop. It runs these in order, then repeats:
    </div>
  );
}

function GetStartedSection() {
  const settings = useStore((s) => s.settings);
  const roblox = useStore((s) => s.roblox);
  const ocrAvailable = useStore((s) => s.ocrAvailable);
  const [choice, setChoice] = useState<boolean | null>(null);
  if (!settings) return null;
  const { done, total, allOk } = setupStatus(settings, roblox, ocrAvailable);
  const open = choice ?? !allOk;
  return (
    <Section
      title="Get started"
      action={
        <div className="flex items-center gap-1.5">
          <Pill tone={allOk ? "ok" : "warn"}>
            {done}/{total} done
          </Pill>
          <Button size="sm" kind="ghost" onClick={() => api.guideOpen()} icon={<BookOpen size={13} />}>
            Guide
          </Button>
          <button
            type="button"
            onClick={() => setChoice(!open)}
            title={open ? "Collapse" : "Expand"}
            className={cx("w-7 h-7 rounded-md grid place-items-center transition-colors", open ? "bg-white/[0.08] text-fg" : "text-fg-mute hover:text-fg")}
          >
            <ChevronDown size={15} className={cx("transition-transform duration-200", open && "rotate-180")} />
          </button>
        </div>
      }
    >
      {open ? (
        <div className="rise border-b border-line">
          <GetStarted />
        </div>
      ) : (
        <button
          type="button"
          onClick={() => setChoice(true)}
          className="w-full text-left px-4 py-3 border-b border-line text-[12px] text-fg-dim hover:bg-white/[0.03]"
        >
          {allOk ? "Everything is set up. Expand to change the server code, sequences or timer area." : `${total - done} setup step${total - done === 1 ? "" : "s"} left. Expand to finish.`}
        </button>
      )}
    </Section>
  );
}

function TimerCard() {
  const timer = useStore((s) => s.timer);
  const settings = useStore((s) => s.settings);
  const state = useStore((s) => s.state);
  const roblox = useStore((s) => s.roblox);
  const ocrAvailable = useStore((s) => s.ocrAvailable);
  const [live, setLive] = useState<{ raw: string; seconds: number | null; error: string | null; estimated: boolean } | null>(null);
  const good = useRef<{ secs: number; at: number } | null>(null);
  const pending = useRef<{ secs: number; at: number } | null>(null);
  const watcherRuns = isActive(state) && state !== "recording" && state !== "test_run";
  const enabled = settings?.timer.enabled ?? false;
  const region = settings?.timer.region;

  useEffect(() => {
    if (watcherRuns || !roblox || !ocrAvailable || !enabled) {
      setLive(null);
      return;
    }
    let alive = true;
    const tick = async () => {
      const now = Date.now();
      const g = good.current && now - good.current.at < 10000 ? good.current : null;
      const predicted = g ? g.secs + Math.round((now - g.at) / 1000) : null;
      try {
        const t = await api.timerTest();
        if (!alive) return;
        let shown: number | null = t.seconds;
        let estimated = false;
        if (t.seconds != null) {
          const fits = predicted == null || Math.abs(t.seconds - predicted) <= 2;
          const p = pending.current;
          const confirmsPending = p != null && Math.abs(t.seconds - (p.secs + Math.round((now - p.at) / 1000))) <= 2;
          if (fits || confirmsPending) {
            good.current = { secs: t.seconds, at: now };
            pending.current = null;
          } else {
            pending.current = { secs: t.seconds, at: now };
            shown = predicted;
            estimated = true;
          }
        } else if (predicted != null) {
          shown = predicted;
          estimated = true;
        }
        setLive({ raw: t.text, seconds: shown, error: null, estimated });
      } catch (e) {
        if (alive) setLive({ raw: "", seconds: predicted, error: String(e), estimated: predicted != null });
      }
    };
    tick();
    const t = setInterval(tick, 1500);
    return () => {
      alive = false;
      clearInterval(t);
    };
  }, [watcherRuns, !!roblox, ocrAvailable, enabled, region?.x, region?.y, region?.w, region?.h]);

  if (!settings) return null;
  const t = settings.timer;
  const secs = watcherRuns ? (timer?.seconds ?? null) : (live?.seconds ?? null);
  const raw = watcherRuns ? (timer?.raw ?? "") : (live?.raw ?? "");
  const parsed = watcherRuns ? (timer?.source === "ocr" && !timer.estimated ? timer.seconds : null) : live && !live.estimated ? live.seconds : null;
  const elapsed = t.mode === "elapsed";
  const pct = secs == null ? 0 : elapsed ? Math.min(1, secs / Math.max(1, t.stop_at_s)) : Math.min(1, t.stop_at_s / Math.max(1, secs));
  const near = secs != null && (elapsed ? secs >= t.stop_at_s - 30 : secs <= t.stop_at_s + 30);
  const src = watcherRuns ? (timer?.source ?? "none") : live?.seconds != null ? "ocr" : "none";
  const estimated = watcherRuns ? !!timer?.estimated : !!live?.estimated;
  const shownRaw = raw.trim().replace(/\s*\n\s*/g, " · ");
  return (
    <div className="px-4 py-3 border-b border-line">
      <div className="flex items-center gap-4">
        <div
          title={estimated ? "Holding the last good reading while the OCR misses a frame" : undefined}
          className={cx("font-mono tabular-nums text-[28px] font-semibold leading-none w-[92px]", secs == null ? "text-fg-mute" : near ? "text-warn" : estimated ? "text-fg-dim" : "text-fg")}
        >
          {secs != null ? fmtClock(secs) : "--:--"}
        </div>
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-1.5 flex-wrap">
            <Pill tone={src === "ocr" ? "ok" : src === "clock" ? "warn" : "mute"}>{src === "ocr" ? "read from screen" : src === "clock" ? "backup clock" : "not reading"}</Pill>
            {watcherRuns && timer?.armed && <Pill tone="accent">watching</Pill>}
            {!watcherRuns && live && <Pill tone="mute">preview</Pill>}
            {estimated && secs != null && <Pill tone="mute">estimated</Pill>}
            {!t.enabled && <Pill tone="bad">disabled</Pill>}
          </div>
          <div className="mt-2 h-1.5 rounded-full bg-white/10 overflow-hidden">
            <div className={cx("h-full rounded-full transition-[width] duration-500", near ? "bg-warn" : "bg-accent")} style={{ width: `${pct * 100}%` }} />
          </div>
          <div className="mt-1 text-[11px] text-fg-mute">
            {elapsed ? "Leaves at" : "Leaves when"} {fmtClock(t.stop_at_s)}
            {elapsed ? "" : " remains"}
          </div>
        </div>
      </div>
      <div className="mt-3 rounded-lg border border-line bg-black/25 px-3 py-2">
        <div className="flex items-center gap-2">
          <span className="text-[10.5px] uppercase tracking-[0.1em] text-fg-mute font-semibold shrink-0">OCR sees</span>
          <span className="ml-auto shrink-0">
            {parsed != null ? <Pill tone="ok">parsed {fmtClock(parsed)}</Pill> : <Pill tone={shownRaw ? "warn" : "mute"}>{shownRaw ? "no time found" : "nothing"}</Pill>}
          </span>
        </div>
        <div className="mt-1 font-mono text-[12px] text-fg-dim break-words select-text min-h-[18px]">
          {live?.error && !watcherRuns ? <span className="text-warn">{live.error}</span> : shownRaw || <span className="text-fg-mute">(no text in the timer area)</span>}
        </div>
        {!roblox && <div className="mt-1 text-[11px] text-fg-mute">Open Roblox to preview the timer.</div>}
        {roblox && !ocrAvailable && <div className="mt-1 text-[11px] text-warn">Windows OCR is not available. Install the English language pack.</div>}
      </div>
    </div>
  );
}

function Stat({ label, value, sub }: { label: string; value: React.ReactNode; sub?: string }) {
  return (
    <div className="flex items-center px-4 h-11 border-b border-line">
      <div className="text-fg-dim">
        {label}
        {sub && <span className="ml-2 text-[11px] text-fg-mute font-mono">{sub}</span>}
      </div>
      <div className="ml-auto font-mono tabular-nums text-[14px] font-medium">{value}</div>
    </div>
  );
}
