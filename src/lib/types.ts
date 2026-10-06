export type RelPoint = { x: number; y: number };
export type RelRect = { x: number; y: number; w: number; h: number };
export type PxPoint = { x: number; y: number };
export type PxRect = { x: number; y: number; w: number; h: number };

export type WindowInfo = { client: PxRect; is_foreground: boolean; visible: boolean; dpi: number };

export type BotState =
  | "stopped"
  | "waiting_for_roblox"
  | "leaving_to_lobby"
  | "rejoining"
  | "waiting_for_spawn"
  | "camera_setup"
  | "farming"
  | "test_run"
  | "recording"
  | "recovering";

export type SeqId = "lobby" | "macro" | "leave" | "buy";
export type StartFrom = "spawn" | "lobby" | "leave" | "first";

export type Job = { kind: "cycle"; from: StartFrom } | { kind: "once"; seq: SeqId; from: number; only: boolean } | { kind: "record"; seq: SeqId };

export type Lifetime = { buys: number; houses: number; laps: number; cycles: number; timer_trips: number; runtime_s: number; sessions: number };

export type Stats = {
  buys: number;
  houses: number;
  laps: number;
  cycles: number;
  timer_trips: number;
  runtime_s: number;
  restarts: number;
  total: Lifetime;
};

export type LogLevel = "debug" | "info" | "warn" | "error";
export type LogLine = { ts: number; level: LogLevel; msg: string };

export type TimerSource = "ocr" | "clock" | "none";
export type TimerReading = { seconds: number | null; source: TimerSource; raw: string; armed: boolean; tripped: boolean; estimated?: boolean };

export type MouseButton = "left" | "right";

export type Action =
  | { kind: "click"; point: RelPoint | null; button: MouseButton }
  | { kind: "key"; key: string; hold_ms: number }
  | { kind: "type"; text: string }
  | { kind: "paste"; text: string }
  | { kind: "scroll"; amount: number; point?: RelPoint | null }
  | { kind: "interact" }
  | { kind: "wait" };

export type RunBlock = { seq: SeqId; enabled?: boolean };

export type Step = Action & { wait_ms: number; enabled?: boolean; note?: string };

export const isOn = (st: Step) => st.enabled !== false;
export type StepKind = Action["kind"];

export type Settings = {
  version: number;
  setup: { click_to_move: boolean; spawn_set: boolean; auto_maximize: boolean };
  server: { code: string; after_join_wait_ms: number };
  keys: { interact: string; equip: string };
  camera: { enabled: boolean; out_steps: number; in_steps: number; step_delay_ms: number; tilt_px: number; settle_ms: number };
  lobby: { steps: Step[] };
  macro: {
    steps: Step[];
    house_wait_ms: number;
    interact_hold_ms: number;
    start_delay_ms: number;
    on_finish: "rejoin" | "repeat";
  };
  leave: { steps: Step[] };
  buy: { steps: Step[]; every: number };
  run_order: RunBlock[];
  timer: {
    enabled: boolean;
    region: RelRect;
    mode: "elapsed" | "remaining";
    stop_at_s: number;
    confirm_reads: number;
    interval_ms: number;
    ocr_scale: number;
    fallback: boolean;
    fallback_after_misses: number;
    fallback_stop_s: number;
  };
  recording: { target: SeqId; mode: "replace" | "append"; record_keys: boolean; camera_first: boolean; collapse_typing: boolean; round_ms: number };
  hotkeys: {
    toggle: string;
    overlay: string;
    quit: string;
    hide_hud: string;
    record: string;
    run_leave: string;
    run_lobby: string;
    run_macro: string;
  };
  ui: { hud_offset: RelPoint; hud_visible: boolean; panel_offset: RelPoint; panel_size: [number, number] };
  watchdog: { enabled: boolean; heartbeat_timeout_s: number; max_restarts: number; restart_backoff_s: number };
  webhook: { enabled: boolean; url: string; every_routes: number; start_stop: boolean; errors: boolean };
  auto_update: boolean;
};

export type Snapshot = {
  state: BotState;
  job: Job | null;
  stats: Stats;
  roblox: WindowInfo | null;
  ocr_available: boolean;
  timer: TimerReading | null;
  settings: Settings;
  version: string;
};

export type OverlayTarget = { kind: "timer_region" } | { kind: "step_point"; seq: SeqId; index: number };

export type OverlaySession = {
  target: OverlayTarget;
  title: string;
  roblox: PxRect;
  overlay_origin: PxPoint;
  region: RelRect | null;
  point: RelPoint | null;
  path: Array<RelPoint | null>;
};

export type RegionPreview = { width: number; height: number; png_base64: string };
export type TimerTest = { text: string; seconds: number | null; png_base64: string };

export const STATE_LABEL: Record<BotState, string> = {
  stopped: "Idle",
  waiting_for_roblox: "Waiting for Roblox",
  leaving_to_lobby: "Leaving to lobby",
  rejoining: "Rejoining server",
  waiting_for_spawn: "Loading in",
  camera_setup: "Setting camera",
  farming: "Visiting houses",
  test_run: "Test run",
  recording: "Recording",
  recovering: "Recovering",
};

export const SEQ_LABEL: Record<SeqId, string> = { lobby: "Lobby", macro: "Macro", leave: "Leave to lobby", buy: "Buy" };

export const SEQ_HOTKEY: Partial<Record<SeqId, keyof Settings["hotkeys"]>> = { lobby: "run_lobby", macro: "run_macro", leave: "run_leave" };

export const seqHotkey = (s: Settings, seq: SeqId): string | null => {
  const k = SEQ_HOTKEY[seq];
  return k ? s.hotkeys[k] : null;
};

export function activeOrder(s: Settings): SeqId[] {
  const order = s.run_order.filter((b) => b.enabled !== false).map((b) => b.seq);
  return order.length ? order : ["lobby", "macro", "leave"];
}

export const CODE_PLACEHOLDER = "{code}";

export function fmtClock(s: number) {
  if (s >= 3600) return `${Math.floor(s / 3600)}:${String(Math.floor((s % 3600) / 60)).padStart(2, "0")}:${String(s % 60).padStart(2, "0")}`;
  return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, "0")}`;
}

export function problems(s: Settings, seq: SeqId): string[] {
  const steps = s[seq].steps.filter(isOn);
  const out: string[] = [];
  if (steps.length === 0) out.push("no steps yet");
  const unset = steps.filter((st) => st.kind === "click" && !st.point).length;
  if (unset) out.push(`${unset} click point${unset > 1 ? "s" : ""} not picked`);
  if (seq === "lobby") {
    if (!s.server.code.trim()) out.push("private server code is empty");
    const code = s.server.code.trim();
    const entersCode = (text: string) => text.includes(CODE_PLACEHOLDER) || (!!code && text.trim() === code);
    if (!steps.some((st) => (st.kind === "type" || st.kind === "paste") && entersCode(st.text))) out.push("never pastes the server code");
  }
  return out;
}
