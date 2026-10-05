import { useEffect, useState, type ReactNode } from "react";
import { ArrowLeft, ArrowRight, Check, DoorOpen, Footprints, Hand, LogIn, MapPin, MousePointerClick, Play, Timer, X } from "lucide-react";
import { api, on } from "../lib/ipc";
import { useStore } from "../lib/store";
import type { Settings } from "../lib/types";
import { fmtClock } from "../lib/types";
import { Button, Kbd, cx } from "../components/primitives";
import logo from "../assets/logo.png";

type GuideStep = { title: string; Icon: typeof Play; body: (s: Settings) => ReactNode; tips: (s: Settings) => ReactNode[] };

const STEPS: GuideStep[] = [
  {
    title: "Switch Roblox to Click to Move",
    Icon: MousePointerClick,
    body: () => "The macro walks by clicking the ground, so Roblox has to move your character where you click.",
    tips: () => [
      <>Open the Roblox menu (<Kbd>Esc</Kbd>) › Settings.</>,
      <>Set <b>Movement Mode</b> to <b>Click to Move</b>.</>,
      <>Keep <b>Camera Mode</b> on Classic, and turn Shift Lock off.</>,
    ],
  },
  {
    title: "Pick a fixed spawn",
    Icon: MapPin,
    body: () => "Every route starts from the exact spot you load in at, so the clicks land in the same place every time.",
    tips: (s) => [
      "Talk to the spawn NPC on the island with the Halloween houses and set your spawn there.",
      "Join your private server and copy its code.",
      <>Paste the code into <b>Home › Get started › Private server code</b> (now: {s.server.code ? <span className="font-mono">{s.server.code}</span> : "not set"}).</>,
    ],
  },
  {
    title: "Record the Macro route",
    Icon: Footprints,
    body: (s) => (
      <>
        Stand at spawn on a fresh load, then press <Kbd>{s.hotkeys.record}</Kbd>. The camera turns to look from above, zooms all the way out,
        and <Kbd>{(s.keys.equip || "?").toUpperCase()}</Kbd> takes out the candy bag, the same as every playback. Then your clicks are recorded like TinyTask.
      </>
    ),
    tips: (s) => [
      "Click the ground to walk to the first house.",
      <>Press <Kbd>{s.keys.interact.toUpperCase()}</Kbd> at the door. The macro holds it, then waits {(s.macro.house_wait_ms / 1000).toFixed(0)}s.</>,
      <>Walk to the next house, press <Kbd>{s.keys.interact.toUpperCase()}</Kbd> again, and so on. Press <Kbd>{s.hotkeys.record}</Kbd> to finish.</>,
    ],
  },
  {
    title: "Record Leave to lobby",
    Icon: DoorOpen,
    body: () => "Two clicks that run the moment the 5:00 listener fires: the menu toggle button that opens the accordion, then Main menu.",
    tips: (s) => [
      <>In <b>Home › Get started › Leave to lobby</b>, press <b>Pick on screen</b> for the menu toggle, then for Main menu.</>,
      <>Test it with <Kbd>{s.hotkeys.run_leave}</Kbd>.</>,
    ],
  },
  {
    title: "Record the Lobby rejoin",
    Icon: LogIn,
    body: () => "From the title screen: click to proceed, Private server join, paste your code into the box, choose Regular, then the First Sea server. The paste uses {code}, so changing the code later just works.",
    tips: (s) => [
      <>In <b>Home › Get started › Lobby</b>, press Record and rejoin by hand once, using <Kbd>Ctrl</Kbd>+<Kbd>V</Kbd> to paste the code.</>,
      `After the last click, the macro waits ${(s.server.after_join_wait_ms / 1000).toFixed(0)}s for the server to load.`,
      <>Test it from the main menu with <Kbd>{s.hotkeys.run_lobby}</Kbd>.</>,
    ],
  },
  {
    title: "Point it at the server timer",
    Icon: Timer,
    body: (s) => (
      <>
        NPCs spawn after 5 minutes. The macro reads the timer in the bottom right and leaves at <b>{fmtClock(s.timer.stop_at_s)}</b>, no matter what it is doing.
      </>
    ),
    tips: (s) => [
      <>Press <Kbd>{s.hotkeys.overlay}</Kbd> and draw a tight box around the timer.</>,
      <>Use <b>Home › Get started › Read now</b> to check it reads the right time.</>,
      `It only stops after ${s.timer.confirm_reads} readings in a row that agree with the real clock, so one bad read never triggers a rejoin.`,
    ],
  },
  {
    title: "Start",
    Icon: Play,
    body: (s) => (
      <>
        Press <Kbd>{s.hotkeys.toggle}</Kbd> anywhere. The loop is: route › timer limit › leave to lobby › rejoin with your code › camera reset › route again.
      </>
    ),
    tips: (s) => [
      <>Press <Kbd>{s.hotkeys.toggle}</Kbd> again to stop at any time.</>,
      "Houses, laps and rejoins are counted on the Dashboard.",
      "Export your settings JSON from Settings › Defaults to share or keep your setup.",
    ],
  },
];

const close = () => api.guideHide();

export default function Guide() {
  const init = useStore((s) => s.init);
  const s = useStore((st) => st.settings);
  const [i, setI] = useState(0);

  useEffect(() => {
    init();
    const un = on("guide:open", () => setI(0));
    return () => {
      un.then((u) => u());
    };
  }, [init]);

  useEffect(() => {
    const h = (e: KeyboardEvent) => {
      if (e.key === "ArrowRight") setI((n) => Math.min(STEPS.length - 1, n + 1));
      if (e.key === "ArrowLeft") setI((n) => Math.max(0, n - 1));
      if (e.key === "Escape") close();
    };
    window.addEventListener("keydown", h);
    return () => window.removeEventListener("keydown", h);
  }, []);

  const step = STEPS[i];
  const last = i === STEPS.length - 1;

  return (
    <div className="h-full w-full p-1.5">
      <div className="glass rounded-2xl h-full w-full flex flex-col overflow-hidden shadow-[0_20px_60px_rgba(0,0,0,0.5)]">
        <header className="h-11 flex items-center gap-2.5 px-4 border-b border-line drag shrink-0">
          <img src={logo} alt="" draggable={false} className="w-6 h-6 rounded-md" />
          <div className="font-semibold">How to set up</div>
          <div className="ml-auto flex items-center gap-1 no-drag">
            {STEPS.map((st, n) => (
              <button
                key={st.title}
                onClick={() => setI(n)}
                title={st.title}
                className={cx(
                  "h-6 min-w-6 px-1.5 rounded-md grid place-items-center text-[11px] font-mono transition-colors",
                  n === i ? "bg-accent text-white" : n < i ? "bg-accent-soft text-accent" : "bg-white/[0.06] text-fg-mute hover:text-fg",
                )}
              >
                {n + 1}
              </button>
            ))}
            <button className="ml-2 w-8 h-8 rounded-lg grid place-items-center text-fg-mute hover:bg-bad-soft hover:text-bad" onClick={close}>
              <X size={15} />
            </button>
          </div>
        </header>

        <main className="flex-1 min-h-0 flex flex-col px-6 pt-5 pb-3">
          {s ? (
            <div key={i} className="rise flex-1 min-h-0 flex flex-col">
              <div className="text-[11px] uppercase tracking-[0.12em] text-fg-mute font-semibold">
                Step {i + 1} of {STEPS.length}
              </div>
              <div className="flex items-center gap-3 mt-2">
                <div className="w-11 h-11 rounded-2xl bg-accent-soft grid place-items-center shrink-0">
                  <step.Icon size={22} className="text-accent" />
                </div>
                <h1 className="text-[18px] font-semibold leading-tight">{step.title}</h1>
              </div>
              <p className="text-fg-dim text-[13px] leading-relaxed mt-3">{step.body(s)}</p>
              <ol className="mt-4 flex flex-col gap-2">
                {step.tips(s).map((t, n) => (
                  <li key={n} className="flex gap-3 items-start rounded-xl border border-line bg-black/25 px-3 py-2.5">
                    <span className="w-5 h-5 rounded-full bg-white/[0.08] grid place-items-center text-[10.5px] font-mono shrink-0 mt-px">{n + 1}</span>
                    <span className="text-[12.5px] leading-relaxed">{t}</span>
                  </li>
                ))}
              </ol>
              {i === 2 && (
                <div className="mt-auto flex items-center gap-2 text-[11px] text-fg-mute">
                  <Hand size={13} className="text-accent" />
                  Each house becomes an <b className="text-fg-dim">Interact</b> step, which you can retime later in Binds › Macro.
                </div>
              )}
            </div>
          ) : (
            <div className="flex-1 grid place-items-center text-fg-dim">Loading…</div>
          )}
        </main>

        <footer className="h-14 flex items-center px-6 border-t border-line shrink-0 gap-2">
          <Button kind="ghost" size="sm" disabled={i === 0} onClick={() => setI(i - 1)} icon={<ArrowLeft size={14} />}>
            Back
          </Button>
          <div className="ml-auto" />
          {last ? (
            <Button kind="primary" size="sm" onClick={close} icon={<Check size={14} />}>
              Finish
            </Button>
          ) : (
            <Button kind="primary" size="sm" onClick={() => setI(i + 1)} icon={<ArrowRight size={14} />}>
              Next
            </Button>
          )}
        </footer>
      </div>
    </div>
  );
}
