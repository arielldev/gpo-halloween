import { ArrowDown, ArrowUp, DoorOpen, Footprints, LogIn, Plus, Repeat, ShoppingBag, Trash2 } from "lucide-react";
import { useStore } from "../lib/store";
import type { SeqId } from "../lib/types";
import { SEQ_LABEL, activeOrder, problems } from "../lib/types";
import { Pill, Toggle, cx } from "./primitives";

export const SEQ_ICON: Record<SeqId, typeof LogIn> = { lobby: LogIn, macro: Footprints, leave: DoorOpen, buy: ShoppingBag };

const ALL: SeqId[] = ["lobby", "macro", "leave", "buy"];

export function RunOrderSummary() {
  const s = useStore((st) => st.settings);
  const goto = useStore((st) => st.goto);
  if (!s) return null;
  const order = activeOrder(s);
  return (
    <button
      type="button"
      onClick={() => goto("binds")}
      title="Edit the F1 run order in Binds"
      className="w-full flex items-center gap-1.5 flex-wrap px-3 py-2 rounded-lg border border-line bg-black/20 hover:bg-white/[0.04] text-left"
    >
      <span className="text-[10.5px] uppercase tracking-[0.1em] text-fg-mute font-semibold mr-1">{s.hotkeys.toggle} runs</span>
      {order.map((q, i) => {
        const Icon = SEQ_ICON[q];
        return (
          <span key={i} className="inline-flex items-center gap-1 text-[12px]">
            {i > 0 && <span className="text-fg-mute">→</span>}
            <Icon size={12} className="text-accent" />
            {SEQ_LABEL[q]}
          </span>
        );
      })}
      <Repeat size={12} className="text-fg-mute ml-1" />
      {s.buy.every > 0 && <span className="text-[11px] text-fg-mute">· Buy every {s.buy.every} routes</span>}
    </button>
  );
}

export function RunOrderEditor() {
  const s = useStore((st) => st.settings);
  const update = useStore((st) => st.update);
  if (!s) return null;
  const blocks = s.run_order;
  const edit = (fn: (b: typeof blocks) => void) => update((x) => fn(x.run_order));
  return (
    <div className="px-4 py-3 border-b border-line">
      <div className="text-[12px] text-fg-dim leading-relaxed mb-2">
        What <b className="text-fg">{s.hotkeys.toggle}</b> runs, top to bottom, then repeats. If the server timer hits its limit during an in-server block, it jumps to
        Leave to lobby and carries on from there.
      </div>
      <div className="rounded-xl border border-line bg-black/20 overflow-hidden">
        {blocks.length === 0 && <div className="px-3 py-3 text-[12px] text-fg-mute">Empty: {s.hotkeys.toggle} uses Lobby → Macro → Leave to lobby.</div>}
        {blocks.map((b, i) => {
          const Icon = SEQ_ICON[b.seq];
          const on = b.enabled !== false;
          const issues = problems(s, b.seq);
          return (
            <div key={i} className={cx("flex items-center gap-2 px-3 py-2 border-b border-line last:border-b-0", !on && "bg-black/20")}>
              <span className="w-5 h-5 rounded-full grid place-items-center font-mono text-[10.5px] bg-white/[0.08] text-fg-dim shrink-0">{i + 1}</span>
              <Icon size={14} className={on ? "text-accent" : "text-fg-mute"} />
              <span className={cx("text-[12.5px] font-medium", !on && "text-fg-mute line-through")}>{SEQ_LABEL[b.seq]}</span>
              {on && (issues.length ? <Pill tone="warn">{issues[0]}</Pill> : <Pill tone="ok">ready</Pill>)}
              <div className="ml-auto flex items-center gap-1 shrink-0">
                <span className="scale-[0.8] origin-right -ml-1.5">
                  <Toggle value={on} onChange={(v) => edit((x) => void (x[i] = { ...x[i], enabled: v ? undefined : false }))} />
                </span>
                <IconBtn title="Move up" disabled={i === 0} onClick={() => edit((x) => void ([x[i - 1], x[i]] = [x[i], x[i - 1]]))}>
                  <ArrowUp size={13} />
                </IconBtn>
                <IconBtn title="Move down" disabled={i === blocks.length - 1} onClick={() => edit((x) => void ([x[i + 1], x[i]] = [x[i], x[i + 1]]))}>
                  <ArrowDown size={13} />
                </IconBtn>
                <button
                  type="button"
                  title="Remove from the run order"
                  onClick={() => edit((x) => void x.splice(i, 1))}
                  className="h-6 w-7 rounded-md grid place-items-center text-bad/80 border border-bad/25 hover:text-bad hover:bg-bad-soft"
                >
                  <Trash2 size={13} />
                </button>
              </div>
            </div>
          );
        })}
        <div className="flex flex-wrap items-center gap-1 px-2 py-2 border-t border-line bg-white/[0.015]">
          <Plus size={13} className="text-fg-mute mx-1" />
          {ALL.map((q) => {
            const Icon = SEQ_ICON[q];
            return (
              <button
                key={q}
                type="button"
                onClick={() => edit((x) => void x.push({ seq: q }))}
                className="h-7 px-2 rounded-md text-[11.5px] text-fg-dim hover:text-fg hover:bg-white/[0.07] inline-flex items-center gap-1"
              >
                <Icon size={12} />
                {SEQ_LABEL[q]}
              </button>
            );
          })}
        </div>
      </div>
    </div>
  );
}

function IconBtn({ children, onClick, disabled, title }: { children: React.ReactNode; onClick: () => void; disabled?: boolean; title: string }) {
  return (
    <button
      type="button"
      title={title}
      disabled={disabled}
      onClick={onClick}
      className="w-7 h-7 rounded-md grid place-items-center text-fg-dim disabled:opacity-25 disabled:pointer-events-none hover:bg-white/[0.08] hover:text-fg"
    >
      {children}
    </button>
  );
}
