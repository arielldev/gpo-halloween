import { useEffect, useRef, useState } from "react";
import { Copy, Trash2 } from "lucide-react";
import { useStore } from "../lib/store";
import type { LogLevel } from "../lib/types";
import { Button, Segmented, cx } from "./primitives";

const LEVEL_COLOR: Record<LogLevel, string> = {
  debug: "text-fg-mute",
  info: "text-fg-dim",
  warn: "text-warn",
  error: "text-bad",
};

export function LogList({ height = 260 }: { height?: number }) {
  const log = useStore((s) => s.log);
  const [filter, setFilter] = useState<"all" | "info" | "warn">("info");
  const ref = useRef<HTMLDivElement>(null);
  const [stick, setStick] = useState(true);

  const visible = log.filter((l) =>
    filter === "all" ? true : filter === "info" ? l.level !== "debug" : l.level === "warn" || l.level === "error",
  );

  useEffect(() => {
    if (stick && ref.current) ref.current.scrollTop = ref.current.scrollHeight;
  }, [visible.length, stick]);

  const copy = () => navigator.clipboard.writeText(visible.map((l) => `${fmt(l.ts)} [${l.level}] ${l.msg}`).join("\n"));

  return (
    <div>
      <div className="flex items-center justify-between px-4 py-2">
        <Segmented
          value={filter}
          onChange={setFilter}
          options={[
            { value: "info", label: "Activity" },
            { value: "warn", label: "Issues" },
            { value: "all", label: "Everything" },
          ]}
        />
        <div className="flex gap-1">
          <Button size="sm" kind="ghost" onClick={copy} icon={<Copy size={13} />} />
          <Button size="sm" kind="ghost" onClick={() => useStore.setState({ log: [] })} icon={<Trash2 size={13} />} />
        </div>
      </div>
      <div
        ref={ref}
        onScroll={(e) => {
          const el = e.currentTarget;
          setStick(el.scrollHeight - el.scrollTop - el.clientHeight < 12);
        }}
        className="font-mono text-[11px] leading-[18px] overflow-y-auto px-4 select-text border-t border-line"
        style={{ height }}
      >
        {visible.length === 0 && <div className="text-fg-mute py-3">Nothing yet.</div>}
        {visible.map((l, i) => (
          <div key={i} className="flex gap-2">
            <span className="text-fg-mute tabular-nums shrink-0">{fmt(l.ts)}</span>
            <span className={cx("break-words", LEVEL_COLOR[l.level])}>{l.msg}</span>
          </div>
        ))}
      </div>
    </div>
  );
}

function fmt(ts: number) {
  const d = new Date(ts);
  return `${String(d.getHours()).padStart(2, "0")}:${String(d.getMinutes()).padStart(2, "0")}:${String(d.getSeconds()).padStart(2, "0")}`;
}
