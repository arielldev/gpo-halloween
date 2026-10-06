import { Crosshair } from "lucide-react";
import { api } from "../lib/ipc";
import { useStore } from "../lib/store";
import type { RelPoint, SeqId } from "../lib/types";
import { cx } from "./primitives";

const CELL =
  "inline-flex items-center gap-1.5 h-7 px-2.5 text-[11.5px] whitespace-nowrap transition-colors outline-none focus-visible:bg-white/[0.1] hover:bg-white/[0.08] disabled:opacity-40 disabled:pointer-events-none";

export function PointField({
  seq,
  index,
  value,
  optional,
}: {
  seq: SeqId;
  index: number;
  value: RelPoint | null;
  optional?: boolean;
}) {
  const roblox = useStore((s) => s.roblox);
  const pick = async () => {
    try {
      await api.overlayOpen({ kind: "step_point", seq, index });
    } catch {
      /* roblox missing the roblox row shows the status */
    }
  };
  return (
    <div
      className={cx(
        "inline-flex items-stretch rounded-lg border bg-white/[0.04] overflow-hidden shadow-[inset_0_1px_0_rgba(255,255,255,0.05)]",
        value || optional ? "border-line-strong" : "border-warn/50",
      )}
    >
      <button
        type="button"
        onClick={pick}
        disabled={!roblox}
        title="Pick on screen"
        className={cx(
          CELL,
          "font-mono tabular-nums",
          value
            ? "text-fg-dim hover:text-fg"
            : optional
              ? "text-fg-mute"
              : "text-warn",
        )}
      >
        <Crosshair size={12} className={value ? "text-accent" : undefined} />
        {value
          ? `${(value.x * 100).toFixed(1)}%, ${(value.y * 100).toFixed(1)}%`
          : optional
            ? "mouse position"
            : "not set"}
      </button>
      <button
        type="button"
        onClick={pick}
        disabled={!roblox}
        className={cx(CELL, "border-l border-line-strong font-medium")}
      >
        {value ? "Re-pick" : optional ? "Pick" : "Pick on screen"}
      </button>
    </div>
  );
}
