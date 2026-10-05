import { create } from "zustand";
import { api, on } from "./ipc";
import type { BotState, Job, LogLine, SeqId, Settings, Stats, TimerReading, WindowInfo } from "./types";

export type Tab = "dashboard" | "binds" | "settings";

type Store = {
  ready: boolean;
  error: string | null;
  state: BotState;
  detail: string | null;
  job: Job | null;
  stats: Stats;
  roblox: WindowInfo | null;
  ocrAvailable: boolean;
  timer: TimerReading | null;
  progress: { seq: SeqId; index: number; total: number } | null;
  recording: { seq: SeqId; steps: number } | null;
  settings: Settings | null;
  version: string;
  log: LogLine[];
  tab: Tab;
  openBind: SeqId | null;
  lastEvent: { kind: string; text: string; ts: number } | null;
  init: () => Promise<void>;
  refresh: () => Promise<void>;
  update: (mutate: (s: Settings) => void) => Promise<void>;
  setSettings: (s: Settings) => void;
  goto: (tab: Tab, bind?: SeqId | null) => void;
};

const EMPTY_STATS: Stats = {
  buys: 0,
  houses: 0,
  laps: 0,
  cycles: 0,
  timer_trips: 0,
  runtime_s: 0,
  restarts: 0,
  total: { buys: 0, houses: 0, laps: 0, cycles: 0, timer_trips: 0, runtime_s: 0, sessions: 0 },
};

let subscribed = false;

const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

export const useStore = create<Store>((set, get) => ({
  ready: false,
  error: null,
  state: "stopped",
  detail: null,
  job: null,
  stats: EMPTY_STATS,
  roblox: null,
  ocrAvailable: false,
  timer: null,
  progress: null,
  recording: null,
  settings: null,
  version: "",
  log: [],
  tab: "dashboard",
  openBind: null,
  lastEvent: null,

  init: async () => {
    if (!subscribed) {
      subscribed = true;
      on("bot:state", (p) =>
        set((s) => ({
          state: p.state,
          detail: p.detail,
          progress: p.state === "stopped" ? null : s.progress,
          recording: p.state === "recording" ? s.recording : null,
          job: p.state === "stopped" ? null : s.job,
        })),
      );
      on("bot:stats", (p) => set({ stats: p }));
      on("bot:log", (p) => set((s) => ({ log: [...s.log.slice(-399), { ts: p.ts, level: p.level, msg: p.msg }] })));
      on("bot:progress", (p) => set({ progress: { seq: p.seq, index: p.index, total: p.total } }));
      on("bot:timer", (p) => {
        set({ timer: { seconds: p.seconds, source: p.source, raw: p.raw, armed: p.armed, tripped: p.tripped, estimated: p.estimated } });
        if (p.tripped) set({ lastEvent: { kind: "timer", text: "Timer limit, rejoining", ts: Date.now() } });
      });
      on("bot:recording", (p) => set({ recording: { seq: p.seq, steps: p.steps } }));
      on("bot:recovery", (p) => set({ lastEvent: { kind: "recovery", text: `Recovered (#${p.attempt})`, ts: Date.now() } }));
      on("roblox:changed", (p) => set({ roblox: p }));
      on("settings:changed", (p) => set({ settings: p }));
    }
    for (let attempt = 0; attempt < 20; attempt++) {
      try {
        await get().refresh();
        return;
      } catch (e) {
        set({ error: String(e) });
        await sleep(250 * Math.min(attempt + 1, 8));
      }
    }
  },

  refresh: async () => {
    const snap = await api.snapshot();
    set({
      ready: true,
      error: null,
      state: snap.state,
      job: snap.job,
      stats: snap.stats,
      roblox: snap.roblox,
      ocrAvailable: snap.ocr_available,
      timer: snap.timer,
      settings: snap.settings,
      version: snap.version,
    });
  },

  update: async (mutate) => {
    const cur = get().settings;
    if (!cur) return;
    const next = structuredClone(cur);
    mutate(next);
    set({ settings: next });
    try {
      await api.settingsSet(next);
    } catch (e) {
      set({ error: String(e) });
    }
  },

  setSettings: (s) => set({ settings: s }),

  goto: (tab, bind) => set({ tab, openBind: bind === undefined ? get().openBind : bind }),
}));

export const isActive = (s: BotState) => s !== "stopped";
