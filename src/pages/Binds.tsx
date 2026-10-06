import { useEffect, useState, type ReactNode } from "react";
import { api } from "../lib/ipc";
import { DoorOpen, Footprints, LogIn } from "lucide-react";
import { useStore } from "../lib/store";
import type { SeqId, Settings } from "../lib/types";
import { SEQ_HOTKEY, SEQ_LABEL, problems, seqHotkey } from "../lib/types";
import { RunOrderEditor, SEQ_ICON } from "../components/RunOrder";
import { StepList, MsField, KeyField } from "../components/StepList";
import { SeqControls, SeqFileButtons, SeqStatus } from "../components/SeqControls";
import { Kbd, KeyCapture, Pill, Row, Section, Segmented, Stepper, Toggle, cx } from "../components/primitives";

const ORDER: SeqId[] = ["lobby", "macro", "leave", "buy"];

const META: Record<SeqId, { Icon: typeof LogIn; sub: string }> = {
  lobby: { Icon: LogIn, sub: "Click to proceed › Private server join › paste code › Regular ×2 › First Sea server." },
  macro: { Icon: Footprints, sub: "Click-to-move from spawn, house to house, pressing interact at each door." },
  leave: { Icon: DoorOpen, sub: "Menu toggle (accordion) › Main menu. Runs when the 5:00 listener fires." },
  buy: { Icon: SEQ_ICON.buy, sub: "Walk to the shop NPC, scroll to the item, buy it, accept. Replaces the Macro every N routes." },
};

export default function Binds() {
  const s = useStore((st) => st.settings);
  const open = useStore((st) => st.openBind);
  const goto = useStore((st) => st.goto);
  if (!s) return null;
  const toggle = (k: SeqId) => goto("binds", open === k ? null : k);

  return (
    <div className="pb-4 pt-2">
      <Section title={`${s.hotkeys.toggle} run order`}>
        <RunOrderEditor />
      </Section>
      <Section title="Sequences">
        {ORDER.map((seq) => (
          <SeqRow key={seq} seq={seq} s={s} open={open === seq} onToggle={() => toggle(seq)} />
        ))}
      </Section>
      <Section title="Global hotkeys">
        <HotkeyList />
      </Section>
    </div>
  );
}

function SeqRow({ seq, s, open, onToggle }: { seq: SeqId; s: Settings; open: boolean; onToggle: () => void }) {
  const update = useStore((st) => st.update);
  const { Icon, sub } = META[seq];
  const issues = problems(s, seq);
  const steps = s[seq].steps;
  const hk = seqHotkey(s, seq);
  const hkKey = SEQ_HOTKEY[seq];

  return (
    <Row
      title={
        <span className="inline-flex items-center gap-2">
          <Icon size={14} className="text-accent" />
          {SEQ_LABEL[seq]}
          <span className="text-fg-mute font-normal text-[11px] font-mono">{steps.length} steps</span>
        </span>
      }
      sub={sub}
      right={
        <>
          <SeqStatus seq={seq} />
          {hk && <Kbd>{hk}</Kbd>}
        </>
      }
      open={open}
      onToggle={onToggle}
    >
      <div className="flex flex-wrap items-center gap-2 mb-3">
        <SeqControls seq={seq} />
        <Segmented
          value={s.recording.mode}
          options={[
            { value: "replace", label: "Replace" },
            { value: "append", label: "Append" },
          ]}
          onChange={(v) => update((x) => void (x.recording.mode = v))}
        />
        <SeqFileButtons seq={seq} />
      </div>

      {seq === "lobby" && <LobbyFields s={s} />}
      {seq === "macro" && <MacroFields s={s} />}
      {seq === "buy" && <BuyFields s={s} />}
      {seq === "leave" && (
        <Hint>
          Pick the menu toggle button that opens the accordion, then the Main menu button. If GPO asks to confirm and you turned that off in game, switch the Confirm
          step off: switched-off steps are skipped and never block Start.
        </Hint>
      )}

      <StepList seq={seq} />

      {issues.length > 0 && (
        <ul className="mt-2 text-[11px] text-warn leading-relaxed">
          {issues.map((i) => (
            <li key={i}>• {i}</li>
          ))}
        </ul>
      )}

      {hkKey && (
        <div className="mt-3 flex items-center gap-3">
          <span className="text-[12px] text-fg-dim w-32">Test-run hotkey</span>
          <KeyCapture value={hk ?? ""} onChange={(v) => update((x) => void (x.hotkeys[hkKey] = v))} />
        </div>
      )}
    </Row>
  );
}

function BuyFields({ s }: { s: Settings }) {
  const update = useStore((st) => st.update);
  return (
    <div className="mb-3 rounded-xl border border-line bg-white/[0.02] px-3 py-1">
      <Field label="Replace the Macro every" hint={s.buy.every > 0 ? `After every ${s.buy.every} routes, the next Macro block runs Buy instead. 0 turns it off.` : "Off. Set how many routes fill your candy bag."}>
        <span className="inline-flex items-center gap-2">
          <Stepper value={s.buy.every} min={0} max={100} onChange={(v) => update((x) => void (x.buy.every = v))} />
          <span className="text-[11px] text-fg-mute">routes</span>
        </span>
      </Field>
      <Hint>
        Starts from your fixed spawn with the same camera and candy bag setup as the Macro. Build it however your shop works: <b>Click</b> steps to walk and pick, a{" "}
        <b>Scroll</b> step to scroll the shop list (pick where to scroll, or leave it at the mouse position), and any <b>Key</b> or <b>Interact</b> steps you need.
        You can also add Buy as its own block in the run order.
      </Hint>
    </div>
  );
}

function LobbyFields({ s }: { s: Settings }) {
  const update = useStore((st) => st.update);
  return (
    <div className="mb-3 rounded-xl border border-line bg-white/[0.02] px-3 py-1">
      <Field label="Wait after join" hint="Time for the server to load before the camera reset">
        <MsField value={s.server.after_join_wait_ms} onChange={(v) => update((x) => void (x.server.after_join_wait_ms = v))} width={72} />
      </Field>
      <Hint>
        Step 1 clicks the screen center to get past "click to proceed". Your private server code lives in the <b>Paste</b> step below: it is put on the clipboard and pasted with
        Ctrl+V. Then <b>Regular</b> is clicked twice (the first click confirms the pasted code) and the <b>First Sea</b> server.
      </Hint>
    </div>
  );
}

function MacroFields({ s }: { s: Settings }) {
  const update = useStore((st) => st.update);
  const cam = s.camera;
  return (
    <div className="mb-3 rounded-xl border border-line bg-white/[0.02] px-3 py-1">
      <Field label="Interact key">
        <KeyField value={s.keys.interact} onChange={(v) => update((x) => void (x.keys.interact = v))} />
      </Field>
      <Field label="Candy bag key" hint="Pressed once after the camera setup so you hold the bag at the doors">
        <KeyField value={s.keys.equip || "off"} onChange={(v) => update((x) => void (x.keys.equip = v))} />
      </Field>
      <Field label="Hold interact">
        <MsField value={s.macro.interact_hold_ms} onChange={(v) => update((x) => void (x.macro.interact_hold_ms = v))} />
      </Field>
      <Field label="Wait at each house">
        <MsField value={s.macro.house_wait_ms} onChange={(v) => update((x) => void (x.macro.house_wait_ms = v))} width={72} />
      </Field>
      <Field label="Delay before route">
        <MsField value={s.macro.start_delay_ms} onChange={(v) => update((x) => void (x.macro.start_delay_ms = v))} />
      </Field>

      <div className="border-t border-line my-1" />
      <Field label="Fixed camera" hint="Third person from above, fully zoomed out, the same before recording and every run">
        <Toggle value={cam.enabled} onChange={(v) => update((x) => void (x.camera.enabled = v))} />
      </Field>
      {cam.enabled && (
        <>
          <Field label="Zoom out steps">
            <Stepper value={cam.out_steps} min={0} max={60} onChange={(v) => update((x) => void (x.camera.out_steps = v))} />
          </Field>
          <Field label="Zoom in steps">
            <Stepper value={cam.in_steps} min={0} max={60} onChange={(v) => update((x) => void (x.camera.in_steps = v))} />
          </Field>
          <Field label="Look from above" hint="Right-drag down in pixels; it stops at the steepest angle. 0 is off.">
            <Stepper value={cam.tilt_px} min={-2000} max={2000} step={50} onChange={(v) => update((x) => void (x.camera.tilt_px = v))} />
          </Field>
        </>
      )}
      <Field label="Camera before recording">
        <Toggle value={s.recording.camera_first} onChange={(v) => update((x) => void (x.recording.camera_first = v))} />
      </Field>
    </div>
  );
}

export function HotkeyList() {
  const s = useStore((st) => st.settings);
  const update = useStore((st) => st.update);
  const [taken, setTaken] = useState<string[]>([]);
  const keysSig = s ? Object.values(s.hotkeys).join("|") : "";
  useEffect(() => {
    const t = setTimeout(() => api.hotkeyConflicts().then(setTaken).catch(() => undefined), 300);
    return () => clearTimeout(t);
  }, [keysSig]);
  if (!s) return null;
  const rows: [keyof Settings["hotkeys"], string][] = [
    ["toggle", "Start / stop macro"],
    ["record", "Record / finish recording"],
    ["run_lobby", "Test run Lobby"],
    ["run_macro", "Test run Macro"],
    ["run_leave", "Test run Leave to lobby"],
    ["overlay", "Edit layout (timer area)"],
    ["hide_hud", "Hide HUD"],
    ["quit", "Quit"],
  ];
  return (
    <>
      {rows.map(([k, label]) => (
        <Row
          key={k}
          title={label}
          sub={taken.includes(s.hotkeys[k]) ? `${s.hotkeys[k]} is also used by another app. The macro listens for it directly; pick another key if it doesn't react.` : undefined}
          right={
            <>
              {taken.includes(s.hotkeys[k]) && <Pill tone="warn">taken</Pill>}
              <KeyCapture value={s.hotkeys[k]} onChange={(v) => update((x) => void (x.hotkeys[k] = v))} />
            </>
          }
        />
      ))}
    </>
  );
}

function Field({ label, hint, children }: { label: string; hint?: string; children: ReactNode }) {
  return (
    <div className="flex items-center min-h-10 py-1 gap-3">
      <div className="min-w-0">
        <div className="text-fg-dim text-[12.5px]">{label}</div>
        {hint && <div className="text-[10.5px] text-fg-mute leading-snug">{hint}</div>}
      </div>
      <div className="ml-auto shrink-0">{children}</div>
    </div>
  );
}

function Hint({ children, className }: { children: ReactNode; className?: string }) {
  return <div className={cx("text-[11px] text-fg-mute leading-relaxed py-2", className)}>{children}</div>;
}
