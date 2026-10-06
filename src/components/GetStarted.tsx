import type { ReactNode } from "react";
import { Play, Square } from "lucide-react";
import { api } from "../lib/ipc";
import { useStore, isActive } from "../lib/store";
import type { Settings, WindowInfo } from "../lib/types";
import { fmtClock, problems } from "../lib/types";
import { Button, Kbd, Pill, Slider, Step, Steps, TextField, Toggle } from "./primitives";
import { TimerArea } from "./TimerArea";
import { KeyField, MsField } from "./StepList";
import { SeqControls, SeqStatus } from "./SeqControls";

export function setupStatus(s: Settings, roblox: WindowInfo | null, ocrAvailable: boolean) {
  const checks = [
    !!roblox,
    s.setup.click_to_move,
    s.setup.spawn_set,
    !!s.server.code.trim(),
    problems(s, "leave").length === 0,
    problems(s, "lobby").length === 0,
    problems(s, "macro").length === 0,
    ocrAvailable,
  ];
  const done = checks.filter(Boolean).length;
  return { done, total: checks.length, allOk: done === checks.length };
}

export function GetStarted() {
  const s = useStore((st) => st.settings);
  const roblox = useStore((st) => st.roblox);
  const ocrAvailable = useStore((st) => st.ocrAvailable);
  const state = useStore((st) => st.state);
  const update = useStore((st) => st.update);
  if (!s) return null;

  const leaveOk = problems(s, "leave").length === 0;
  const lobbyOk = problems(s, "lobby").length === 0;
  const macroOk = problems(s, "macro").length === 0;
  const codeOk = !!s.server.code.trim();
  const allOk = !!roblox && s.setup.click_to_move && s.setup.spawn_set && codeOk && leaveOk && lobbyOk && macroOk && ocrAvailable;
  const active = isActive(state);
  const canStart = codeOk && leaveOk && lobbyOk && macroOk;

  return (
    <div className="px-4 pt-4 pb-2">
          <Steps>
            <Step
              n={1}
              done={!!roblox}
              title="Open Roblox and join GPO"
              sub={roblox ? `Detected · ${roblox.client.w} × ${roblox.client.h}. The macro keeps Roblox maximized in a window (not fullscreen) so clicks and the timer area always line up.` : "The macro attaches to the Roblox window automatically."}
            >
              <Pill tone={roblox ? "ok" : "warn"}>{roblox ? "detected" : "not found"}</Pill>
              <Check label="Keep Roblox maximized" value={s.setup.auto_maximize} onChange={(v) => update((x) => void (x.setup.auto_maximize = v))} />
            </Step>

            <Step
              n={2}
              done={s.setup.click_to_move}
              title="Switch movement to Click to Move"
              sub={<>Roblox menu (<Kbd>Esc</Kbd>) › Settings › Movement Mode › <b>Click to Move</b>. Turn Shift Lock off.</>}
            >
              <Check label="Done" value={s.setup.click_to_move} onChange={(v) => update((x) => void (x.setup.click_to_move = v))} />
            </Step>

            <Step
              n={3}
              done={s.setup.spawn_set}
              title="Set your spawn at the island NPC"
              sub="Talk to the spawn NPC on the island with the Halloween houses. Every run starts from this exact spot."
            >
              <Check label="Spawn set" value={s.setup.spawn_set} onChange={(v) => update((x) => void (x.setup.spawn_set = v))} />
            </Step>

            <Step
              n={4}
              done={codeOk}
              title={
                <span className="inline-flex items-center gap-2">
                  Private server code <Pill tone={codeOk ? "ok" : "bad"}>required</Pill>
                </span>
              }
              sub="Copy your private server code here. Every rejoin pastes it into GPO's code box."
            >
              <div className="w-[190px]">
                <TextField value={s.server.code} onChange={(v) => update((x) => void (x.server.code = v.trim()))} placeholder="paste your server code" mono />
              </div>
              <Labeled label="load wait">
                <MsField value={s.server.after_join_wait_ms} onChange={(v) => update((x) => void (x.server.after_join_wait_ms = v))} width={64} />
              </Labeled>
            </Step>

            <Step
              n={5}
              done={leaveOk}
              title={<SeqTitle label="Leave to lobby" seq={<SeqStatus seq="leave" />} />}
              sub="2 clicks: the menu toggle button (opens the accordion), then Main menu. Runs the moment the 5:00 timer listener fires."
            >
              <SeqControls seq="leave" editLink />
            </Step>

            <Step
              n={6}
              done={lobbyOk}
              title={<SeqTitle label="Lobby: rejoin your server" seq={<SeqStatus seq="lobby" />} />}
              sub={
                <>
                  Click to proceed › <b>Private server join</b> › code box, paste with <Kbd>Ctrl</Kbd>+<Kbd>V</Kbd> › <b>Regular</b> twice (the first click confirms the code) ›{" "}
                  <b>First Sea</b> server.
                  Pick each point, or press Record and do it once by hand.
                </>
              }
            >
              <SeqControls seq="lobby" editLink />
            </Step>

            <Step
              n={7}
              done={macroOk}
              title={<SeqTitle label="Macro: house route" seq={<SeqStatus seq="macro" />} />}
              sub={
                <>
                  Stand at spawn on a fresh load and press Record. The camera turns to look from above, zooms all the way out, and{" "}
                  <Kbd>{(s.keys.equip || "?").toUpperCase()}</Kbd> takes out the candy bag. Then click to walk to each house and press{" "}
                  <Kbd>{s.keys.interact.toUpperCase()}</Kbd> at each door.
                </>
              }
            >
              <Labeled label="interact">
                <KeyField value={s.keys.interact} onChange={(v) => update((x) => void (x.keys.interact = v))} />
              </Labeled>
              <Labeled label="candy bag">
                <KeyField value={s.keys.equip || "off"} onChange={(v) => update((x) => void (x.keys.equip = v))} />
              </Labeled>
              <Labeled label="wait per house">
                <MsField value={s.macro.house_wait_ms} onChange={(v) => update((x) => void (x.macro.house_wait_ms = v))} width={60} />
              </Labeled>
              <div className="basis-full h-0" />
              <SeqControls seq="macro" editLink />
            </Step>

            <Step
              n={8}
              done={ocrAvailable}
              title="Server timer area"
              sub={
                <>
                  Press <Kbd>{s.hotkeys.overlay}</Kbd> to lay out the box over the bottom-right timer, then use Read now to check it.
                  {!ocrAvailable && " Windows OCR is missing: install the English language pack."}
                </>
              }
            >
              <div className="basis-full">
                <TimerArea />
              </div>
              <Labeled label="leave at">
                <Slider value={s.timer.stop_at_s} min={60} max={600} step={5} width={170} format={fmtClock} onChange={(v) => update((x) => void (x.timer.stop_at_s = v))} />
              </Labeled>
            </Step>

            <Step
              n={9}
              last
              done={allOk}
              title="Start"
              sub={
                <>
                  Stand at your spawn in the private server and press <Kbd>{s.hotkeys.toggle}</Kbd>. It farms the houses, leaves before{" "}
                  {fmtClock(s.timer.stop_at_s)}, rejoins with your code, and repeats.
                </>
              }
            >
              {active ? (
                <Button kind="danger" size="sm" onClick={() => api.botStop()} icon={<Square size={12} />}>
                  Stop
                </Button>
              ) : (
                <Button kind="primary" size="sm" disabled={!canStart} onClick={() => api.botStart()} icon={<Play size={13} />}>
                  Start
                  <span className="font-mono text-[10px] opacity-75">{s.hotkeys.toggle}</span>
                </Button>
              )}
              {!canStart && <span className="text-[11px] text-fg-mute">Needs the server code and steps 5–7.</span>}
            </Step>
          </Steps>
    </div>
  );
}

function SeqTitle({ label, seq }: { label: string; seq: ReactNode }) {
  return (
    <span className="inline-flex items-center gap-2 flex-wrap">
      {label}
      {seq}
    </span>
  );
}

function Labeled({ label, children }: { label: string; children: ReactNode }) {
  return (
    <span className="inline-flex items-center gap-1.5">
      <span className="text-[11px] text-fg-mute whitespace-nowrap">{label}</span>
      {children}
    </span>
  );
}

function Check({ label, value, onChange }: { label: string; value: boolean; onChange: (v: boolean) => void }) {
  return (
    <span className="inline-flex items-center gap-2">
      <Toggle value={value} onChange={onChange} />
      <span className="text-[12px] text-fg-dim">{label}</span>
    </span>
  );
}
