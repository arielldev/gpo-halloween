import { useEffect, useRef, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { House, Keyboard, Minus, Settings2, X } from "lucide-react";
import { api, on } from "../lib/ipc";
import { useStore, type Tab } from "../lib/store";
import { cx, Dot } from "../components/primitives";
import { ConnectGate } from "../components/ConnectGate";
import logo from "../assets/logo.png";
import Dashboard from "../pages/Dashboard";
import Binds from "../pages/Binds";
import SettingsPage from "../pages/Settings";

const TABS: { id: Tab; label: string; icon: React.ReactNode }[] = [
  { id: "dashboard", label: "Home", icon: <House size={17} /> },
  { id: "binds", label: "Binds", icon: <Keyboard size={17} /> },
  { id: "settings", label: "Settings", icon: <Settings2 size={17} /> },
];

export default function Panel() {
  const init = useStore((s) => s.init);
  const ready = useStore((s) => s.ready);
  const error = useStore((s) => s.error);
  const roblox = useStore((s) => s.roblox);
  const version = useStore((s) => s.version);
  const tab = useStore((s) => s.tab);
  const goto = useStore((s) => s.goto);
  const [skipGate, setSkipGate] = useState(false);
  const [appear, setAppear] = useState(0);
  const visibleRef = useRef(true);

  useEffect(() => {
    const un = on("ui:visibility", (v) => {
      if (v && !visibleRef.current) setAppear((n) => n + 1);
      visibleRef.current = v;
    });
    return () => {
      un.then((u) => u());
    };
  }, []);

  useEffect(() => {
    init().finally(() => api.panelShow());
  }, [init]);

  useEffect(() => {
    if (roblox) setSkipGate(false);
  }, [roblox]);

  useEffect(() => {
    let t: number | undefined;
    const schedule = () => {
      window.clearTimeout(t);
      t = window.setTimeout(() => api.panelPlacementChanged(), 400);
    };
    const w = getCurrentWindow();
    const subs = [w.onResized(schedule), w.onMoved(schedule)];
    return () => {
      subs.forEach((p) => p.then((u) => u()));
    };
  }, []);

  const gated = ready && !roblox && !skipGate;

  return (
    <div className="h-full w-full p-1.5">
      <div key={appear} className="glass rounded-2xl h-full w-full flex overflow-hidden shadow-[0_20px_60px_rgba(0,0,0,0.5)] rise">
        <nav className={cx("w-14 shrink-0 flex flex-col items-center py-3 border-r border-line bg-black/20", gated && "opacity-40 pointer-events-none")}>
          <img src={logo} alt="" draggable={false} className="w-9 h-9 rounded-xl mb-4 drag shadow-[0_2px_10px_rgba(0,0,0,0.4)]" />
          {TABS.map((t) => (
            <button
              key={t.id}
              onClick={() => goto(t.id)}
              title={t.label}
              className={cx(
                "w-10 h-10 rounded-xl grid place-items-center mb-1 transition-colors",
                tab === t.id ? "bg-white/[0.1] text-fg" : "text-fg-mute hover:text-fg hover:bg-white/[0.05]",
              )}
            >
              {t.icon}
            </button>
          ))}
          <div className="mt-auto flex flex-col items-center gap-1.5 text-[10px] text-fg-mute">
            <Dot tone={roblox ? (roblox.is_foreground ? "ok" : "accent") : "mute"} />
            <span className="font-mono">{version}</span>
          </div>
        </nav>
        <div className="flex-1 min-w-0 flex flex-col">
          <header className="h-11 flex items-center px-4 border-b border-line drag shrink-0">
            <div className="font-semibold">{gated ? "GPO Halloween" : TABS.find((t) => t.id === tab)?.label}</div>
            <div className="ml-auto flex items-center gap-1 no-drag">
              <button className="w-8 h-8 rounded-lg grid place-items-center text-fg-mute hover:bg-white/[0.06] hover:text-fg" onClick={() => getCurrentWindow().minimize()}>
                <Minus size={15} />
              </button>
              <button className="w-8 h-8 rounded-lg grid place-items-center text-fg-mute hover:bg-bad-soft hover:text-bad" onClick={() => api.panelHide()}>
                <X size={15} />
              </button>
            </div>
          </header>
          <main className="flex-1 overflow-y-auto">
            {!ready && <Booting error={error} />}
            {gated && <ConnectGate onSkip={() => setSkipGate(true)} />}
            {ready && !gated && (
              <>
                {tab === "dashboard" && <Dashboard />}
                {tab === "binds" && <Binds />}
                {tab === "settings" && <SettingsPage />}
              </>
            )}
          </main>
        </div>
      </div>
    </div>
  );
}

function Booting({ error }: { error: string | null }) {
  return (
    <div className="h-full flex flex-col items-center justify-center text-center px-8">
      <div className="w-8 h-8 rounded-full border-2 border-white/10 border-t-accent animate-spin mb-4" />
      <div className="text-fg-dim">Starting…</div>
      {error && <div className="mt-3 text-[11px] text-warn font-mono break-all select-text max-w-[320px]">{error}</div>}
    </div>
  );
}
