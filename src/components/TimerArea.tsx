import { useEffect, useState } from "react";
import { Crosshair, RefreshCw, ScanText } from "lucide-react";
import { api } from "../lib/ipc";
import { useStore } from "../lib/store";
import type { TimerTest } from "../lib/types";
import { fmtClock } from "../lib/types";
import { Button, Pill, cx } from "./primitives";

export function TimerArea() {
  const roblox = useStore((s) => s.roblox);
  const ocrAvailable = useStore((s) => s.ocrAvailable);
  const s = useStore((st) => st.settings);
  const [test, setTest] = useState<TimerTest | null>(null);
  const [png, setPng] = useState<string | null>(null);
  const [err, setErr] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const region = s?.timer.region;

  const preview = async () => {
    if (!roblox || !region) return;
    try {
      setPng((await api.regionPreview(region, 240)).png_base64);
      setErr(null);
    } catch (e) {
      setErr(String(e));
    }
  };

  useEffect(() => {
    preview();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [region?.x, region?.y, region?.w, region?.h, roblox?.client.w, roblox?.client.h]);

  const read = async () => {
    setBusy(true);
    try {
      const t = await api.timerTest();
      setTest(t);
      setPng(t.png_base64);
      setErr(null);
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  };

  if (!s || !region) return null;
  return (
    <div className="flex gap-3">
      <div className="relative rounded-lg overflow-hidden border border-line-strong bg-black/40 shrink-0 grid place-items-center" style={{ width: 132, height: 72 }}>
        {png ? (
          <img src={`data:image/png;base64,${png}`} alt="" className="w-full h-full object-contain" />
        ) : (
          <div className="text-fg-mute text-[11px] text-center px-2">{roblox ? "no preview" : "open Roblox"}</div>
        )}
      </div>
      <div className="flex-1 min-w-0 flex flex-col gap-2">
        <div className="font-mono text-[11px] text-fg-dim tabular-nums">
          {(region.x * 100).toFixed(1)}%, {(region.y * 100).toFixed(1)}% · {(region.w * 100).toFixed(1)}% × {(region.h * 100).toFixed(1)}%
        </div>
        {test && (
          <div className="flex items-center gap-2 text-[11px] min-w-0">
            <Pill tone={test.seconds != null ? "ok" : "warn"}>{test.seconds != null ? `read ${fmtClock(test.seconds)}` : "no time found"}</Pill>
            <span className="font-mono text-fg-mute truncate select-text">{test.text || "(nothing)"}</span>
          </div>
        )}
        {err && <div className="text-[11px] text-warn leading-snug">{err}</div>}
        <div className="flex gap-2 mt-auto flex-wrap">
          <Button size="sm" disabled={!roblox} onClick={() => api.overlayOpen({ kind: "timer_region" })} icon={<Crosshair size={13} />}>
            Draw
          </Button>
          <Button size="sm" disabled={!roblox || !ocrAvailable || busy} onClick={read} icon={<ScanText size={13} />}>
            Read now
          </Button>
          <Button size="sm" kind="ghost" disabled={!roblox} onClick={preview} icon={<RefreshCw size={13} className={cx(busy && "animate-spin")} />} />
        </div>
      </div>
    </div>
  );
}
