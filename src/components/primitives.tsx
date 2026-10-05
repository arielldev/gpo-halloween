import { useEffect, useRef, useState, type ReactNode } from "react";
import { ChevronDown } from "lucide-react";

export function cx(...parts: Array<string | false | null | undefined>) {
  return parts.filter(Boolean).join(" ");
}

export function Row({
  title,
  sub,
  right,
  children,
  open,
  onToggle,
  disabled,
}: {
  title: ReactNode;
  sub?: ReactNode;
  right?: ReactNode;
  children?: ReactNode;
  open?: boolean;
  onToggle?: () => void;
  disabled?: boolean;
}) {
  const expandable = children != null && onToggle != null;
  return (
    <div className={cx("border-b border-line", disabled && "opacity-50")}>
      <div
        className={cx(
          "flex items-center gap-3 px-4 py-3 min-h-[52px] outline-none",
          expandable && "cursor-pointer hover:bg-white/[0.03] transition-colors focus-visible:bg-white/[0.04]",
          expandable && open && "bg-white/[0.02]",
        )}
        role={expandable ? "button" : undefined}
        tabIndex={expandable ? 0 : undefined}
        aria-expanded={expandable ? open : undefined}
        onClick={expandable ? onToggle : undefined}
        onKeyDown={
          expandable
            ? (e) => {
                if (e.target === e.currentTarget && (e.key === "Enter" || e.key === " ")) {
                  e.preventDefault();
                  onToggle();
                }
              }
            : undefined
        }
      >
        <div className="flex-1 min-w-0">
          <div className="font-medium leading-tight truncate">{title}</div>
          {sub && <div className="text-fg-dim text-[12px] leading-snug mt-0.5">{sub}</div>}
        </div>
        {right && (
          <div className="shrink-0 flex items-center gap-2" onClick={(e) => e.stopPropagation()}>
            {right}
          </div>
        )}
        {expandable && (
          <span className={cx("w-6 h-6 rounded-md grid place-items-center shrink-0 transition-colors", open ? "bg-white/[0.08] text-fg" : "text-fg-mute")}>
            <ChevronDown size={15} className={cx("transition-transform duration-200", open && "rotate-180")} />
          </span>
        )}
      </div>
      {expandable && open && <div className="px-4 pt-2 pb-4 rise">{children}</div>}
      {!expandable && children && <div className="px-4 pt-2 pb-4">{children}</div>}
    </div>
  );
}

export function Section({ title, children, action }: { title: string; children: ReactNode; action?: ReactNode }) {
  return (
    <div className="mb-6 first:pt-1">
      <div className="flex items-center justify-between px-4 pb-2 pt-3">
        <div className="text-[11px] uppercase tracking-[0.12em] text-fg-mute font-semibold">{title}</div>
        {action}
      </div>
      <div className="border-t border-line">{children}</div>
    </div>
  );
}

export function Steps({ children }: { children: ReactNode }) {
  return <ol className="flex flex-col">{children}</ol>;
}

export function Step({
  n,
  title,
  sub,
  done,
  last,
  children,
}: {
  n: number;
  title: ReactNode;
  sub?: ReactNode;
  done?: boolean;
  last?: boolean;
  children?: ReactNode;
}) {
  return (
    <li className="relative flex gap-3 pb-4 last:pb-0">
      {!last && <span className="absolute left-[11px] top-6 bottom-0 w-px bg-line-strong" />}
      <span
        className={cx(
          "relative z-10 w-[23px] h-[23px] shrink-0 rounded-full grid place-items-center text-[11px] font-semibold tabular-nums border",
          done ? "bg-accent border-accent text-white" : "bg-bg-elev border-line-strong text-fg-dim",
        )}
      >
        {n}
      </span>
      <div className="flex-1 min-w-0 pt-0.5">
        <div className="font-medium leading-tight">{title}</div>
        {sub && <div className="text-fg-dim text-[12px] leading-snug mt-0.5">{sub}</div>}
        {children && <div className="mt-2 flex items-center gap-2 flex-wrap">{children}</div>}
      </div>
    </li>
  );
}

export function Toggle({ value, onChange, disabled }: { value: boolean; onChange: (v: boolean) => void; disabled?: boolean }) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={value}
      disabled={disabled}
      onClick={() => onChange(!value)}
      className={cx(
        "relative shrink-0 rounded-full border-0 p-0 transition-colors duration-200 outline-none",
        "focus-visible:ring-2 focus-visible:ring-accent/60",
        value ? "bg-accent" : "bg-white/15",
        disabled && "opacity-40",
      )}
      style={{ width: 38, height: 22 }}
    >
      <span
        className="absolute rounded-full bg-white transition-transform duration-200"
        style={{
          width: 16,
          height: 16,
          top: 3,
          left: 3,
          transform: value ? "translateX(16px)" : "translateX(0)",
          boxShadow: "0 1px 2px rgba(0,0,0,0.35)",
        }}
      />
    </button>
  );
}

export function Button({
  children,
  onClick,
  kind = "default",
  size = "md",
  disabled,
  className,
  icon,
}: {
  children?: ReactNode;
  onClick?: () => void;
  kind?: "default" | "primary" | "danger" | "ghost";
  size?: "sm" | "md";
  disabled?: boolean;
  className?: string;
  icon?: ReactNode;
}) {
  const base =
    "inline-flex items-center justify-center gap-1.5 rounded-lg font-medium select-none whitespace-nowrap transition-[background-color,border-color,box-shadow,transform,color,filter] duration-150 outline-none focus-visible:ring-2 focus-visible:ring-accent/60 focus-visible:ring-offset-2 focus-visible:ring-offset-bg-elev hover:-translate-y-px active:translate-y-0 active:scale-[0.97] disabled:opacity-40 disabled:pointer-events-none";
  const sizes = size === "sm" ? "h-8 px-3 text-[12px]" : "h-9 px-3.5 text-[13px]";
  const kinds = {
    default: "bg-white/[0.06] hover:bg-white/[0.1] border border-line-strong hover:border-white/20 shadow-[inset_0_1px_0_rgba(255,255,255,0.06),0_1px_2px_rgba(0,0,0,0.3)]",
    primary: "bg-accent text-white hover:brightness-110 shadow-[inset_0_1px_0_rgba(255,255,255,0.18),0_4px_14px_rgba(255,122,26,0.35)]",
    danger: "bg-bad-soft text-bad hover:bg-bad/25 border border-bad/30",
    ghost: "hover:bg-white/[0.06] text-fg-dim hover:text-fg",
  }[kind];
  return (
    <button type="button" disabled={disabled} onClick={onClick} className={cx(base, sizes, kinds, className)}>
      {icon}
      {children}
    </button>
  );
}

export function Slider({
  value,
  min,
  max,
  step = 1,
  onChange,
  format = (v) => String(v),
  width = 180,
}: {
  value: number;
  min: number;
  max: number;
  step?: number;
  onChange: (v: number) => void;
  format?: (v: number) => string;
  width?: number;
}) {
  const [local, setLocal] = useState(value);
  useEffect(() => setLocal(value), [value]);
  const pct = ((local - min) / (max - min)) * 100;
  return (
    <div className="flex items-center gap-3" style={{ width }}>
      <div className="relative flex-1 h-5 flex items-center">
        <div className="absolute inset-x-0 h-[3px] rounded-full bg-white/10" />
        <div className="absolute left-0 h-[3px] rounded-full bg-accent" style={{ width: `${pct}%` }} />
        <input
          type="range"
          min={min}
          max={max}
          step={step}
          value={local}
          onChange={(e) => setLocal(Number(e.target.value))}
          onPointerUp={() => onChange(local)}
          onKeyUp={() => onChange(local)}
          className="absolute inset-0 w-full opacity-0 cursor-pointer"
        />
        <div
          className="absolute w-3.5 h-3.5 rounded-full bg-white shadow pointer-events-none -translate-x-1/2"
          style={{ left: `${pct}%` }}
        />
      </div>
      <div className="font-mono text-[12px] text-fg-dim tabular-nums w-14 text-right">{format(local)}</div>
    </div>
  );
}

export function Stepper({
  value,
  min,
  max,
  step = 1,
  onChange,
  suffix,
}: {
  value: number;
  min: number;
  max: number;
  step?: number;
  onChange: (v: number) => void;
  suffix?: string;
}) {
  const clamp = (v: number) => Math.min(max, Math.max(min, v));
  return (
    <div className="inline-flex items-center rounded-lg border border-line-strong bg-white/[0.04] overflow-hidden">
      <button
        type="button"
        className="w-8 h-8 hover:bg-white/[0.08] text-fg-dim active:bg-white/[0.12]"
        onClick={() => onChange(clamp(value - step))}
      >
        −
      </button>
      <input
        type="number"
        value={value}
        onChange={(e) => onChange(clamp(Number(e.target.value) || 0))}
        className="w-14 h-8 bg-transparent text-center font-mono text-[12px] tabular-nums [appearance:textfield] [&::-webkit-inner-spin-button]:appearance-none"
      />
      {suffix && <span className="text-fg-mute text-[11px] pr-1">{suffix}</span>}
      <button
        type="button"
        className="w-8 h-8 hover:bg-white/[0.08] text-fg-dim active:bg-white/[0.12]"
        onClick={() => onChange(clamp(value + step))}
      >
        +
      </button>
    </div>
  );
}

export function Segmented<T extends string>({
  value,
  options,
  onChange,
}: {
  value: T;
  options: { value: T; label: ReactNode }[];
  onChange: (v: T) => void;
}) {
  return (
    <div className="inline-flex rounded-lg bg-black/25 border border-line p-0.5 gap-0.5" role="radiogroup">
      {options.map((o) => (
        <button
          key={o.value}
          type="button"
          role="radio"
          aria-checked={o.value === value}
          onClick={() => onChange(o.value)}
          className={cx(
            "px-3 h-7 rounded-md text-[12px] font-medium whitespace-nowrap select-none transition-all duration-150 outline-none focus-visible:ring-2 focus-visible:ring-accent/60 active:scale-[0.97]",
            o.value === value
              ? "bg-white/[0.12] text-fg shadow-[0_1px_2px_rgba(0,0,0,0.4),inset_0_1px_0_rgba(255,255,255,0.08)]"
              : "text-fg-dim hover:text-fg hover:bg-white/[0.05]",
          )}
        >
          {o.label}
        </button>
      ))}
    </div>
  );
}

export function KeyCapture({ value, onChange, single }: { value: string; onChange: (v: string) => void; single?: boolean }) {
  const [listening, setListening] = useState(false);
  const ref = useRef<HTMLButtonElement>(null);
  useEffect(() => {
    if (!listening) return;
    const h = (e: KeyboardEvent) => {
      e.preventDefault();
      e.stopPropagation();
      if (e.key === "Escape") {
        setListening(false);
        return;
      }
      if (single) {
        if (e.key.length === 1) {
          onChange(e.key.toLowerCase());
          setListening(false);
        }
        return;
      }
      if (["Control", "Shift", "Alt", "Meta"].includes(e.key)) return;
      if (e.key.length > 1 && !/^(F\d{1,2}|Tab|Space|Enter|Insert|Delete|Home|End|PageUp|PageDown|Arrow(Up|Down|Left|Right))$/.test(e.key)) return;
      const mods = [e.ctrlKey && "Ctrl", e.shiftKey && "Shift", e.altKey && "Alt"].filter(Boolean);
      const key = e.key.length === 1 ? e.key.toUpperCase() : e.key;
      onChange([...mods, key].join("+"));
      setListening(false);
    };
    window.addEventListener("keydown", h, true);
    return () => window.removeEventListener("keydown", h, true);
  }, [listening, onChange, single]);
  return (
    <button
      ref={ref}
      type="button"
      onClick={() => setListening(true)}
      onBlur={() => setListening(false)}
      className={cx(
        "h-8 min-w-[72px] px-3 rounded-lg font-mono text-[12px] border transition-colors",
        listening
          ? "border-accent bg-accent-soft text-accent animate-pulse"
          : "border-line-strong bg-white/[0.04] hover:bg-white/[0.08]",
      )}
    >
      {listening ? "press key…" : value.toUpperCase()}
    </button>
  );
}

export function TextField({
  value,
  onChange,
  placeholder,
  type = "text",
  mono,
  className,
}: {
  value: string;
  onChange: (v: string) => void;
  placeholder?: string;
  type?: "text" | "url";
  mono?: boolean;
  className?: string;
}) {
  const [local, setLocal] = useState(value);
  useEffect(() => setLocal(value), [value]);
  return (
    <input
      type={type}
      value={local}
      placeholder={placeholder}
      onChange={(e) => setLocal(e.target.value)}
      onBlur={() => local !== value && onChange(local)}
      onKeyDown={(e) => e.key === "Enter" && (e.target as HTMLInputElement).blur()}
      className={cx(
        "h-8 px-3 rounded-lg bg-white/[0.04] border border-line-strong focus:border-accent/70 text-[12px] w-full select-text",
        mono && "font-mono",
        className,
      )}
    />
  );
}

export function Pill({ children, tone = "mute" }: { children: ReactNode; tone?: "ok" | "warn" | "bad" | "accent" | "mute" | "fruit" }) {
  const tones = {
    ok: "bg-ok-soft text-ok",
    warn: "bg-warn-soft text-warn",
    bad: "bg-bad-soft text-bad",
    accent: "bg-accent-soft text-accent",
    mute: "bg-white/[0.06] text-fg-dim",
    fruit: "bg-fruit/15 text-fruit",
  }[tone];
  return <span className={cx("inline-flex items-center h-5 px-2 rounded-full text-[11px] font-medium", tones)}>{children}</span>;
}

export function Dot({ tone, pulse }: { tone: "ok" | "warn" | "bad" | "accent" | "mute"; pulse?: boolean }) {
  const c = { ok: "text-ok bg-ok", warn: "text-warn bg-warn", bad: "text-bad bg-bad", accent: "text-accent bg-accent", mute: "text-fg-mute bg-fg-mute" }[tone];
  return <span className={cx("inline-block w-2 h-2 rounded-full", c, pulse && "pulse")} />;
}

export function Kbd({ children }: { children: ReactNode }) {
  return (
    <kbd className="inline-flex items-center align-middle h-5 px-1.5 mx-0.5 rounded-md border border-line-strong bg-white/[0.06] font-mono text-[10.5px] text-fg leading-none whitespace-nowrap">
      {children}
    </kbd>
  );
}

export function fmtRuntime(s: number) {
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  const sec = s % 60;
  return `${String(h).padStart(2, "0")}:${String(m).padStart(2, "0")}:${String(sec).padStart(2, "0")}`;
}
