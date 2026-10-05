import { Camera, Circle, DoorOpen, FlaskConical, Hourglass, LogIn, MonitorOff, Power, RefreshCw, Footprints } from "lucide-react";
import type { BotState } from "../lib/types";
import { cx } from "./primitives";

export type StateTone = "ok" | "warn" | "mute" | "rec" | "accent";

const MAP: Record<BotState, { Icon: typeof Power; tone: StateTone; spin?: boolean }> = {
  stopped: { Icon: Power, tone: "mute" },
  waiting_for_roblox: { Icon: MonitorOff, tone: "warn" },
  leaving_to_lobby: { Icon: DoorOpen, tone: "accent" },
  rejoining: { Icon: LogIn, tone: "accent" },
  waiting_for_spawn: { Icon: Hourglass, tone: "accent" },
  camera_setup: { Icon: Camera, tone: "ok" },
  farming: { Icon: Footprints, tone: "ok" },
  test_run: { Icon: FlaskConical, tone: "accent" },
  recording: { Icon: Circle, tone: "rec" },
  recovering: { Icon: RefreshCw, tone: "warn", spin: true },
};

const TEXT: Record<StateTone, string> = { ok: "text-ok", warn: "text-warn", mute: "text-fg-mute", rec: "text-rec", accent: "text-accent" };
const BG: Record<StateTone, string> = { ok: "bg-ok-soft", warn: "bg-warn-soft", mute: "bg-white/[0.06]", rec: "bg-rec-soft", accent: "bg-accent-soft" };

export function stateTone(state: BotState): StateTone {
  return MAP[state].tone;
}

export function StateIcon({ state, size = 16, className }: { state: BotState; size?: number; className?: string }) {
  const { Icon, tone, spin } = MAP[state];
  return (
    <Icon
      size={size}
      className={cx(TEXT[tone], spin && "animate-spin [animation-duration:2.4s]", state === "recording" && "fill-current pulse", className)}
    />
  );
}

export function StateBadge({ state, size = 44, icon = 20, className }: { state: BotState; size?: number; icon?: number; className?: string }) {
  const tone = stateTone(state);
  return (
    <div className={cx("rounded-full grid place-items-center shrink-0", BG[tone], className)} style={{ width: size, height: size }}>
      <StateIcon state={state} size={icon} />
    </div>
  );
}
