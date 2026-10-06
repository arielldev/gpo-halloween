import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  BotState,
  LogLine,
  OverlaySession,
  OverlayTarget,
  PxRect,
  RegionPreview,
  RelPoint,
  RelRect,
  SeqId,
  Settings,
  Snapshot,
  Stats,
  TimerReading,
  TimerTest,
  WindowInfo,
} from "./types";

export const api = {
  snapshot: () => invoke<Snapshot>("snapshot"),
  botStart: () => invoke<void>("bot_start"),
  botStop: () => invoke<void>("bot_stop"),
  botToggle: () => invoke<void>("bot_toggle"),
  runOnce: (seq: SeqId, from?: number, only?: boolean) => invoke<void>("run_once", { seq, from: from ?? null, only: only ?? null }),
  recordToggle: (seq?: SeqId) => invoke<void>("record_toggle", { seq: seq ?? null }),
  settingsGet: () => invoke<Settings>("settings_get"),
  settingsSet: (settings: Settings) => invoke<void>("settings_set", { settings }),
  settingsReset: () => invoke<Settings>("settings_reset"),
  settingsJson: () => invoke<string>("settings_json"),
  settingsExport: () => invoke<string>("settings_export"),
  settingsImport: (json: string) => invoke<Settings>("settings_import", { json }),
  sequenceExport: (seq: SeqId) => invoke<string>("sequence_export", { seq }),
  sequenceImport: (seq: SeqId, json: string, append: boolean) =>
    invoke<{ settings: Settings; count: number; from: SeqId | null }>("sequence_import", { seq, json, append }),
  sequencesDir: () => invoke<string>("sequences_dir"),
  presetList: () => invoke<string[]>("preset_list"),
  presetSave: (name: string) => invoke<void>("preset_save", { name }),
  presetLoad: (name: string) => invoke<Settings>("preset_load", { name }),
  presetDelete: (name: string) => invoke<void>("preset_delete", { name }),
  overlayOpen: (target: OverlayTarget) => invoke<OverlaySession>("overlay_open", { target }),
  overlayCommit: (commit: { target: OverlayTarget; region?: RelRect | null; point?: RelPoint | null }) =>
    invoke<Settings>("overlay_commit", { commit }),
  overlayCancel: () => invoke<void>("overlay_cancel"),
  overlayPending: () => invoke<OverlaySession | null>("overlay_pending"),
  panelPlacementChanged: () => invoke<void>("panel_placement_changed"),
  regionPreview: (region: RelRect, maxDim = 320) => invoke<RegionPreview>("region_preview", { region, maxDim }),
  timerTest: (region?: RelRect) => invoke<TimerTest>("timer_test", { region: region ?? null }),
  panelVisible: () => invoke<boolean>("panel_visible"),
  hudSetOffset: (offset: RelPoint) => invoke<void>("hud_set_offset", { offset }),
  hudToggle: () => invoke<boolean>("hud_toggle"),
  panelShow: () => invoke<void>("panel_show"),
  panelHide: () => invoke<void>("panel_hide"),
  panelToggle: () => invoke<void>("panel_toggle"),
  guideOpen: () => invoke<void>("guide_open"),
  guideHide: () => invoke<void>("guide_hide"),
  quit: () => invoke<void>("app_quit"),
  openUrl: (url: string) => invoke<void>("open_url", { url }),
  openPath: (path: string) => invoke<void>("open_path", { path }),
  dataDir: () => invoke<string>("data_dir"),
  hotkeyConflicts: () => invoke<string[]>("hotkey_conflicts"),
  webhookTest: () => invoke<void>("webhook_test"),
};

type Events = {
  "bot:state": { kind: "state"; state: BotState; detail: string | null };
  "bot:stats": { kind: "stats" } & Stats;
  "bot:log": { kind: "log" } & LogLine;
  "bot:progress": { kind: "progress"; seq: SeqId; index: number; total: number };
  "bot:timer": { kind: "timer" } & TimerReading;
  "bot:recording": { kind: "recording"; seq: SeqId; steps: number };
  "bot:recovery": { kind: "recovery"; attempt: number; reason: string };
  "roblox:changed": WindowInfo | null;
  "overlay:roblox": PxRect;
  "overlay:session": OverlaySession;
  "overlay:close": null;
  "ui:visibility": boolean;
  "guide:open": null;
  "settings:changed": Settings;
  "panel:visible": boolean;
};

export function on<K extends keyof Events>(name: K, cb: (payload: Events[K]) => void): Promise<UnlistenFn> {
  return listen<Events[K]>(name, (e) => cb(e.payload));
}
