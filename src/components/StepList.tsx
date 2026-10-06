import { useEffect, useState } from "react";
import { ArrowDown, ArrowUp, ChevronsRight, Clipboard, Clock, Hand, Keyboard, MousePointerClick, Mouse, Play, Plus, Trash2, Type, X } from "lucide-react";
import { api } from "../lib/ipc";
import { useStore, isActive } from "../lib/store";
import type { SeqId, Settings, Step, StepKind } from "../lib/types";
import { CODE_PLACEHOLDER } from "../lib/types";
import { isOn } from "../lib/types";
import { PointField } from "./PointField";
import { Button, Kbd, Segmented, Stepper, TextField, Toggle, cx } from "./primitives";

const KIND: Record<StepKind, { label: string; Icon: typeof Clock }> = {
  click: { label: "Click", Icon: MousePointerClick },
  key: { label: "Key", Icon: Keyboard },
  type: { label: "Type", Icon: Type },
  paste: { label: "Paste", Icon: Clipboard },
  scroll: { label: "Scroll", Icon: Mouse },
  interact: { label: "Interact", Icon: Hand },
  wait: { label: "Wait", Icon: Clock },
};

const ADD_ORDER: StepKind[] = ["click", "interact", "paste", "key", "type", "wait", "scroll"];

function blank(kind: StepKind, s: Settings): Step {
  switch (kind) {
    case "click":
      return { kind, point: null, button: "left", wait_ms: 1000 };
    case "key":
      return { kind, key: "e", hold_ms: 60, wait_ms: 300 };
    case "type":
      return { kind, text: "", wait_ms: 300 };
    case "paste":
      return { kind, text: CODE_PLACEHOLDER, wait_ms: 600 };
    case "scroll":
      return { kind, amount: -5, point: null, wait_ms: 200 };
    case "interact":
      return { kind, wait_ms: 0 };
    case "wait":
      return { kind, wait_ms: s.macro.house_wait_ms };
  }
}

export function StepList({ seq }: { seq: SeqId }) {
  const s = useStore((st) => st.settings);
  const update = useStore((st) => st.update);
  const state = useStore((st) => st.state);
  const recording = useStore((st) => st.recording);
  const progress = useStore((st) => st.progress);
  const [confirmClear, setConfirmClear] = useState(false);
  if (!s) return null;
  const steps = s[seq].steps;
  const live = state !== "stopped" && progress?.seq === seq ? progress.index : -1;
  const isRecording = state === "recording" && recording?.seq === seq;

  const edit = (mutate: (steps: Step[]) => void) => update((x) => mutate(x[seq].steps));
  const patch = (i: number, p: Partial<Step>) => edit((st) => void (st[i] = { ...st[i], ...p } as Step));
  const move = (i: number, d: number) =>
    edit((st) => {
      const j = i + d;
      if (j < 0 || j >= st.length) return;
      [st[i], st[j]] = [st[j], st[i]];
    });

  return (
    <div className="rounded-xl border border-line bg-black/20 overflow-hidden">
      {isRecording && (
        <div className="px-3 py-2 bg-rec-soft text-rec text-[12px] font-medium flex items-center gap-2 border-b border-rec/20">
          <span className="w-2 h-2 rounded-full bg-rec pulse text-rec" />
          Recording… {recording?.steps ?? 0} steps captured. Press <Kbd>{s.hotkeys.record}</Kbd> to finish.
        </div>
      )}
      {steps.length === 0 && !isRecording && (
        <div className="px-3 py-4 text-[12px] text-fg-mute text-center">No steps yet. Record them or add them below.</div>
      )}
      <ol>
        {steps.map((st, i) => (
          <StepRow
            key={i}
            seq={seq}
            index={i}
            step={st}
            s={s}
            live={live === i}
            first={i === 0}
            last={i === steps.length - 1}
            onPatch={(p) => patch(i, p)}
            onMove={(d) => move(i, d)}
            onDelete={() => edit((x) => void x.splice(i, 1))}
          />
        ))}
      </ol>
      <div className="flex flex-wrap items-center gap-1 px-2 py-2 border-t border-line bg-white/[0.015]">
        <Plus size={13} className="text-fg-mute mx-1" />
        {ADD_ORDER.map((k) => {
          const { label, Icon } = KIND[k];
          return (
            <button
              key={k}
              type="button"
              onClick={() => edit((x) => void x.push(blank(k, s)))}
              className="h-7 px-2 rounded-md text-[11.5px] text-fg-dim hover:text-fg hover:bg-white/[0.07] inline-flex items-center gap-1"
            >
              <Icon size={12} />
              {label}
            </button>
          );
        })}
        {steps.length > 0 && (
          <div className="ml-auto">
            {confirmClear ? (
              <span className="inline-flex gap-1">
                <Button size="sm" kind="danger" onClick={() => { edit((x) => void x.splice(0)); setConfirmClear(false); }}>
                  Clear all
                </Button>
                <Button size="sm" kind="ghost" onClick={() => setConfirmClear(false)} icon={<X size={13} />} />
              </span>
            ) : (
              <Button size="sm" kind="ghost" onClick={() => setConfirmClear(true)} icon={<Trash2 size={13} />} />
            )}
          </div>
        )}
      </div>
    </div>
  );
}

function StepRow({
  seq,
  index,
  step,
  s,
  live,
  first,
  last,
  onPatch,
  onMove,
  onDelete,
}: {
  seq: SeqId;
  index: number;
  step: Step;
  s: Settings;
  live: boolean;
  first: boolean;
  last: boolean;
  onPatch: (p: Partial<Step>) => void;
  onMove: (d: number) => void;
  onDelete: () => void;
}) {
  const { label, Icon } = KIND[step.kind];
  return (
    <li className={cx("px-3 py-2.5 border-b border-line last:border-b-0", live && "bg-accent-soft", !isOn(step) && "bg-black/20")}>
      <div className="flex items-center gap-2 h-6">
        <span className={cx("w-5 h-5 rounded-full grid place-items-center font-mono text-[10.5px] tabular-nums shrink-0", live ? "bg-accent text-white" : "bg-white/[0.08] text-fg-dim")}>
          {index + 1}
        </span>
        <span className="inline-flex items-center gap-1.5 text-[12.5px] font-semibold whitespace-nowrap">
          <Icon size={13} className="text-accent" />
          {label}
        </span>
        {step.note && <span className={cx("text-[11.5px] truncate min-w-0", isOn(step) ? "text-fg-mute" : "text-fg-mute line-through")}>· {step.note}</span>}
        {!isOn(step) && <span className="text-[10.5px] uppercase tracking-wide text-fg-mute shrink-0">skipped</span>}
        <div className="ml-auto flex items-center gap-1.5 shrink-0">
          <RunButtons seq={seq} index={index} on={isOn(step)} />
          <span title={isOn(step) ? "Step on: runs" : "Step off: skipped"} className="scale-[0.8] origin-right -ml-1.5">
            <Toggle value={isOn(step)} onChange={(v) => onPatch({ enabled: v ? undefined : false })} />
          </span>
          <button
            type="button"
            title={`Delete step ${index + 1}`}
            onClick={onDelete}
            className="h-6 w-7 rounded-md grid place-items-center text-bad/80 border border-bad/25 hover:text-bad hover:bg-bad-soft"
          >
            <Trash2 size={13} />
          </button>
        </div>
      </div>
      <div className={cx("flex flex-wrap items-center gap-2 mt-2 pl-7", !isOn(step) && "opacity-45")}>
        <StepEditor seq={seq} index={index} step={step} s={s} onPatch={onPatch} />
      </div>
      <div className={cx("flex items-center gap-2 mt-2 pl-7", !isOn(step) && "opacity-45")}>
        <span className="text-[11px] text-fg-mute shrink-0 whitespace-nowrap">{step.kind === "interact" ? "extra wait" : step.kind === "wait" ? "wait" : seq === "macro" && step.kind === "click" ? "walk time" : "then wait"}</span>
        <MsField value={step.wait_ms} onChange={(v) => onPatch({ wait_ms: v })} />
        <div className="flex-1 min-w-0">
          <TextField value={step.note ?? ""} onChange={(v) => onPatch({ note: v })} placeholder="note (optional)" className="h-7! text-[11.5px]!" />
        </div>
        <IconBtn disabled={first} onClick={() => onMove(-1)} title="Move up">
          <ArrowUp size={13} />
        </IconBtn>
        <IconBtn disabled={last} onClick={() => onMove(1)} title="Move down">
          <ArrowDown size={13} />
        </IconBtn>
      </div>
    </li>
  );
}

function StepEditor({ seq, index, step, s, onPatch }: { seq: SeqId; index: number; step: Step; s: Settings; onPatch: (p: Partial<Step>) => void }) {
  switch (step.kind) {
    case "click":
      return (
        <>
          <PointField seq={seq} index={index} value={step.point} />
          <Segmented
            value={step.button}
            options={[
              { value: "left", label: "L" },
              { value: "right", label: "R" },
            ]}
            onChange={(v) => onPatch({ button: v } as Partial<Step>)}
          />
        </>
      );
    case "key":
      return (
        <>
          <KeyField value={step.key} onChange={(v) => onPatch({ key: v } as Partial<Step>)} />
          <span className="text-[11px] text-fg-mute">hold</span>
          <MsField value={step.hold_ms} onChange={(v) => onPatch({ hold_ms: v } as Partial<Step>)} />
        </>
      );
    case "paste": {
      const isCode = step.text.includes(CODE_PLACEHOLDER) || (!!s.server.code && step.text.trim() === s.server.code.trim());
      if (isCode) return <ServerCodeField seq={seq} index={index} />;
      return (
        <div className="flex-1 min-w-[140px]">
          <TextField value={step.text} onChange={(v) => onPatch({ text: v } as Partial<Step>)} placeholder="text to paste" mono className="h-7!" />
        </div>
      );
    }
    case "type":
      return (
        <div className="flex-1 min-w-[160px]">
          <TextField value={step.text} onChange={(v) => onPatch({ text: v } as Partial<Step>)} placeholder={CODE_PLACEHOLDER} mono className="h-7!" />
        </div>
      );
    case "scroll":
      return (
        <>
          <Segmented
            value={step.amount < 0 ? "down" : "up"}
            options={[
              { value: "down", label: "Down" },
              { value: "up", label: "Up" },
            ]}
            onChange={(v) => onPatch({ amount: (v === "down" ? -1 : 1) * Math.max(1, Math.abs(step.amount)) } as Partial<Step>)}
          />
          <Stepper
            value={Math.max(1, Math.abs(step.amount))}
            min={1}
            max={60}
            onChange={(v) => onPatch({ amount: (step.amount < 0 ? -1 : 1) * Math.max(1, v) } as Partial<Step>)}
          />
          <span className="text-[11px] text-fg-mute whitespace-nowrap">notches</span>
          <span className="inline-flex items-center gap-1">
            <span className="text-[11px] text-fg-mute whitespace-nowrap">at</span>
            <PointField seq={seq} index={index} value={step.point ?? null} optional />
            {step.point && (
              <button
                type="button"
                title="Clear: scroll wherever the mouse is"
                onClick={() => onPatch({ point: null } as Partial<Step>)}
                className="w-6 h-6 rounded-md grid place-items-center text-fg-mute hover:text-fg hover:bg-white/[0.08]"
              >
                <X size={12} />
              </button>
            )}
          </span>
        </>
      );
    case "interact":
      return (
        <span className="text-[12px] text-fg-dim">
          hold <Kbd>{s.keys.interact.toUpperCase()}</Kbd> {s.macro.interact_hold_ms} ms, then wait {(s.macro.house_wait_ms / 1000).toFixed(1)} s
        </span>
      );
    case "wait":
      return <span className="text-[12px] text-fg-dim">pause for the time below</span>;
  }
}

function ServerCodeField({ seq, index }: { seq: SeqId; index: number }) {
  const code = useStore((st) => st.settings?.server.code ?? "");
  const update = useStore((st) => st.update);
  return (
    <>
      <div className="flex-1 min-w-[150px]">
        <TextField
          value={code}
          onChange={(v) =>
            update((x) => {
              x.server.code = v.trim();
              const st = x[seq].steps[index];
              if (st && st.kind === "paste") st.text = CODE_PLACEHOLDER;
            })
          }
          placeholder="paste your private server code"
          mono
          className={cx("h-7!", !code && "border-bad/60!")}
        />
      </div>
      <span className="text-[11px] text-fg-mute whitespace-nowrap">server code · Ctrl+V</span>
    </>
  );
}

function RunButtons({ seq, index, on }: { seq: SeqId; index: number; on: boolean }) {
  const busy = useStore((st) => isActive(st.state));
  const roblox = useStore((st) => st.roblox);
  const off = busy || !roblox;
  return (
    <span className="inline-flex items-center rounded-md border border-line-strong bg-white/[0.03] mr-0.5">
      <button
        type="button"
        title={on ? `Run step ${index + 1} only` : "Switch this step on to run it"}
        disabled={off || !on}
        onClick={() => api.runOnce(seq, index, true)}
        className="w-7 h-6 grid place-items-center text-ok hover:bg-ok-soft rounded-l-md disabled:opacity-30 disabled:pointer-events-none"
      >
        <Play size={12} />
      </button>
      <button
        type="button"
        title={`Run from step ${index + 1} to the end`}
        disabled={off}
        onClick={() => api.runOnce(seq, index, false)}
        className="w-7 h-6 grid place-items-center text-accent hover:bg-accent-soft rounded-r-md border-l border-line-strong disabled:opacity-30 disabled:pointer-events-none"
      >
        <ChevronsRight size={13} />
      </button>
    </span>
  );
}

function IconBtn({ children, onClick, disabled, title }: { children: React.ReactNode; onClick: () => void; disabled?: boolean; title: string }) {
  return (
    <button
      type="button"
      title={title}
      disabled={disabled}
      onClick={onClick}
      className={cx(
        "w-7 h-7 rounded-md grid place-items-center text-fg-dim disabled:opacity-25 disabled:pointer-events-none shrink-0 hover:bg-white/[0.08] hover:text-fg",
      )}
    >
      {children}
    </button>
  );
}

export function MsField({ value, onChange, width = 64 }: { value: number; onChange: (v: number) => void; width?: number }) {
  const [local, setLocal] = useState(String(value));
  useEffect(() => setLocal(String(value)), [value]);
  const commit = () => {
    const n = Math.max(0, Math.min(600000, Math.round(Number(local) || 0)));
    setLocal(String(n));
    if (n !== value) onChange(n);
  };
  return (
    <span className="inline-flex items-center h-7 rounded-md border border-line-strong bg-white/[0.04] focus-within:border-accent/70 shrink-0">
      <input
        type="number"
        value={local}
        onChange={(e) => setLocal(e.target.value)}
        onBlur={commit}
        onKeyDown={(e) => e.key === "Enter" && (e.target as HTMLInputElement).blur()}
        className="bg-transparent text-right font-mono text-[11.5px] tabular-nums px-1.5 [appearance:textfield] [&::-webkit-inner-spin-button]:appearance-none select-text"
        style={{ width }}
      />
      <span className="text-[10.5px] text-fg-mute pr-1.5">ms</span>
    </span>
  );
}

const NAMED: Record<string, string> = {
  Escape: "Escape",
  Enter: "Enter",
  Tab: "Tab",
  " ": "Space",
  Backspace: "Backspace",
  Delete: "Delete",
  Shift: "Shift",
  Control: "Ctrl",
};

export function KeyField({ value, onChange }: { value: string; onChange: (v: string) => void }) {
  const [listening, setListening] = useState(false);
  useEffect(() => {
    if (!listening) return;
    const h = (e: KeyboardEvent) => {
      e.preventDefault();
      e.stopPropagation();
      const named = NAMED[e.key];
      if (named) onChange(named);
      else if (e.key.length === 1) onChange(e.key.toLowerCase());
      else return;
      setListening(false);
    };
    window.addEventListener("keydown", h, true);
    return () => window.removeEventListener("keydown", h, true);
  }, [listening, onChange]);
  return (
    <button
      type="button"
      onClick={() => setListening(true)}
      onBlur={() => setListening(false)}
      className={cx(
        "h-7 min-w-[56px] px-2 rounded-md font-mono text-[11.5px] border transition-colors",
        listening ? "border-accent bg-accent-soft text-accent animate-pulse" : "border-line-strong bg-white/[0.04] hover:bg-white/[0.08]",
      )}
    >
      {listening ? "press…" : value.length === 1 ? value.toUpperCase() : value}
    </button>
  );
}
