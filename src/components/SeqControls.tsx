import { useState } from "react";
import { Circle, FileDown, FileUp, FlaskConical, ListOrdered, Square } from "lucide-react";
import { api } from "../lib/ipc";
import { useStore, isActive } from "../lib/store";
import type { SeqId } from "../lib/types";
import { SEQ_LABEL, problems, seqHotkey } from "../lib/types";
import { Button, Pill } from "./primitives";

export function SeqStatus({ seq }: { seq: SeqId }) {
  const s = useStore((st) => st.settings);
  const state = useStore((st) => st.state);
  const recording = useStore((st) => st.recording);
  if (!s) return null;
  if (state === "recording" && recording?.seq === seq) return <Pill tone="bad">recording · {recording.steps}</Pill>;
  const issues = problems(s, seq);
  return issues.length ? <Pill tone="warn">{issues[0]}</Pill> : <Pill tone="ok">ready · {s[seq].steps.length} steps</Pill>;
}

export function SeqControls({ seq, editLink }: { seq: SeqId; editLink?: boolean }) {
  const s = useStore((st) => st.settings);
  const state = useStore((st) => st.state);
  const recording = useStore((st) => st.recording);
  const goto = useStore((st) => st.goto);
  if (!s) return null;
  const recThis = state === "recording" && recording?.seq === seq;
  const busy = isActive(state);
  const hk = seqHotkey(s, seq);
  return (
    <>
      <Button
        size="sm"
        kind={recThis ? "danger" : "primary"}
        disabled={busy && !recThis}
        onClick={() => api.recordToggle(seq)}
        icon={recThis ? <Square size={12} /> : <Circle size={12} className="fill-current" />}
      >
        {recThis ? "Stop recording" : "Record"}
        <span className="font-mono text-[10px] opacity-75">{s.hotkeys.record}</span>
      </Button>
      <Button size="sm" disabled={busy || s[seq].steps.length === 0} onClick={() => api.runOnce(seq)} icon={<FlaskConical size={13} />}>
        Test
        {hk && <span className="font-mono text-[10px] opacity-75">{hk}</span>}
      </Button>
      {editLink && (
        <Button size="sm" kind="ghost" onClick={() => goto("binds", seq)} icon={<ListOrdered size={13} />}>
          Edit steps
        </Button>
      )}
    </>
  );
}

export function SeqFileButtons({ seq }: { seq: SeqId }) {
  const s = useStore((st) => st.settings);
  const state = useStore((st) => st.state);
  const setSettings = useStore((st) => st.setSettings);
  const [msg, setMsg] = useState<{ ok: boolean; text: string } | null>(null);
  if (!s) return null;
  const busy = isActive(state);
  const append = s.recording.mode === "append";

  const exportSeq = async () => {
    try {
      const path = await api.sequenceExport(seq);
      setMsg({ ok: true, text: `Saved ${path.split(/[\\/]/).pop()}` });
      api.openPath(await api.sequencesDir()).catch(() => undefined);
    } catch (e) {
      setMsg({ ok: false, text: String(e) });
    }
  };

  const importSeq = () => {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = ".json,application/json";
    input.onchange = async () => {
      const f = input.files?.[0];
      if (!f) return;
      try {
        const r = await api.sequenceImport(seq, await f.text(), append);
        setSettings(r.settings);
        const from = r.from && r.from !== seq ? ` (exported from ${SEQ_LABEL[r.from]})` : "";
        setMsg({ ok: true, text: `${append ? "Added" : "Loaded"} ${r.count} steps from ${f.name}${from}` });
      } catch (e) {
        setMsg({ ok: false, text: String(e) });
      }
    };
    input.click();
  };

  return (
    <>
      <Button size="sm" kind="ghost" disabled={s[seq].steps.length === 0} onClick={exportSeq} icon={<FileDown size={13} />}>
        Export
      </Button>
      <Button size="sm" kind="ghost" disabled={busy} onClick={importSeq} icon={<FileUp size={13} />}>
        Import
      </Button>
      {msg && (
        <span className="basis-full">
          <Pill tone={msg.ok ? "ok" : "warn"}>
            <span className="truncate max-w-[330px] select-text">{msg.text}</span>
          </Pill>
        </span>
      )}
    </>
  );
}
