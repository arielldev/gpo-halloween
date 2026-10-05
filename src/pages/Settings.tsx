import { useEffect, useState } from "react";
import { Copy, FileDown, FolderOpen, RotateCcw, Save, Trash2, Upload } from "lucide-react";
import { api } from "../lib/ipc";
import { useStore } from "../lib/store";
import { fmtClock } from "../lib/types";
import { Button, Pill, Row, Section, Segmented, Slider, Stepper, TextField, Toggle } from "../components/primitives";
import { HotkeyList } from "./Binds";

export default function SettingsPage() {
  const s = useStore((st) => st.settings);
  const update = useStore((st) => st.update);
  const setSettings = useStore((st) => st.setSettings);
  const version = useStore((st) => st.version);
  const [open, setOpen] = useState<string | null>(null);
  const [presets, setPresets] = useState<string[]>([]);
  const [presetName, setPresetName] = useState("");
  const [msg, setMsg] = useState<string | null>(null);
  const [dataDir, setDataDir] = useState("");
  const [confirmReset, setConfirmReset] = useState(false);

  useEffect(() => {
    api.presetList().then(setPresets);
    api.dataDir().then(setDataDir);
  }, []);

  if (!s) return null;
  const toggle = (k: string) => setOpen((o) => (o === k ? null : k));
  const t = s.timer;

  const savePreset = async () => {
    if (!presetName.trim()) return;
    try {
      await api.presetSave(presetName.trim());
      setPresets(await api.presetList());
      setPresetName("");
    } catch (e) {
      setMsg(String(e));
    }
  };

  const exportJson = async () => {
    try {
      const path = await api.settingsExport();
      setMsg(`Saved ${path}`);
    } catch (e) {
      setMsg(String(e));
    }
  };

  const copyJson = async () => {
    try {
      await navigator.clipboard.writeText(await api.settingsJson());
      setMsg("Settings JSON copied to clipboard");
    } catch (e) {
      setMsg(String(e));
    }
  };

  const importJson = () => {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = ".json";
    input.onchange = async () => {
      const f = input.files?.[0];
      if (!f) return;
      try {
        setSettings(await api.settingsImport(await f.text()));
        setMsg(`Imported ${f.name}`);
      } catch (e) {
        setMsg(String(e));
      }
    };
    input.click();
  };

  return (
    <div className="pb-4 pt-2">
      <Section title="Server timer">
        <Row
          title="Timer watch"
          sub={`Reads the bottom-right timer and leaves at ${fmtClock(t.stop_at_s)}, whatever the macro is doing.`}
          right={<Toggle value={t.enabled} onChange={(v) => update((x) => void (x.timer.enabled = v))} />}
          open={open === "timer"}
          onToggle={() => toggle("timer")}
        >
          <Field label="Timer counts">
            <Segmented
              value={t.mode}
              options={[
                { value: "elapsed", label: "Up (elapsed)" },
                { value: "remaining", label: "Down (remaining)" },
              ]}
              onChange={(v) => update((x) => void (x.timer.mode = v))}
            />
          </Field>
          <Field label={t.mode === "elapsed" ? "Leave at" : "Leave when left"}>
            <Slider value={t.stop_at_s} min={10} max={600} step={5} format={fmtClock} onChange={(v) => update((x) => void (x.timer.stop_at_s = v))} />
          </Field>
          <Field label="Confirm reads">
            <Stepper value={t.confirm_reads} min={2} max={10} onChange={(v) => update((x) => void (x.timer.confirm_reads = v))} />
          </Field>
          <div className="text-[11px] text-fg-mute leading-relaxed pb-2">
            Only readings that move in step with the real clock count, so a single misread like 1:50 → 4:50 never triggers a rejoin.
          </div>
          <Field label="Read every">
            <Slider value={t.interval_ms} min={250} max={3000} step={250} format={(v) => `${(v / 1000).toFixed(2)} s`} onChange={(v) => update((x) => void (x.timer.interval_ms = v))} />
          </Field>
          <Field label="OCR upscale">
            <Stepper value={t.ocr_scale} min={1} max={6} onChange={(v) => update((x) => void (x.timer.ocr_scale = v))} />
          </Field>
        </Row>
        <Row
          title="Backup clock"
          sub="If the timer cannot be read, count from the moment you joined instead."
          right={<Toggle value={t.fallback} onChange={(v) => update((x) => void (x.timer.fallback = v))} />}
          open={open === "fallback"}
          onToggle={() => toggle("fallback")}
        >
          <Field label="After failed reads">
            <Stepper value={t.fallback_after_misses} min={3} max={30} onChange={(v) => update((x) => void (x.timer.fallback_after_misses = v))} />
          </Field>
          <Field label="Leave at">
            <Slider value={t.fallback_stop_s} min={30} max={600} step={5} format={fmtClock} onChange={(v) => update((x) => void (x.timer.fallback_stop_s = v))} />
          </Field>
        </Row>
      </Section>

      <Section title="Recording">
        <Row title="Record key presses" sub="Off records only clicks and the interact key." right={<Toggle value={s.recording.record_keys} onChange={(v) => update((x) => void (x.recording.record_keys = v))} />} />
        <Row title="Turn typing into one step" sub="Typed letters become a Type step, and your server code becomes {code}." right={<Toggle value={s.recording.collapse_typing} onChange={(v) => update((x) => void (x.recording.collapse_typing = v))} />} />
        <Row title="Round waits to" right={<Segmented value={String(s.recording.round_ms)} options={[{ value: "10", label: "10 ms" }, { value: "50", label: "50 ms" }, { value: "100", label: "100 ms" }]} onChange={(v) => update((x) => void (x.recording.round_ms = Number(v)))} />} />
      </Section>

      <Section title="Safety">
        <Row
          title="Watchdog"
          sub="If a step hangs, leaves to the lobby and starts the loop again."
          right={<Toggle value={s.watchdog.enabled} onChange={(v) => update((x) => void (x.watchdog.enabled = v))} />}
          open={open === "wd"}
          onToggle={() => toggle("wd")}
        >
          <Field label="Max restarts">
            <Stepper value={s.watchdog.max_restarts} min={1} max={50} onChange={(v) => update((x) => void (x.watchdog.max_restarts = v))} />
          </Field>
          <Field label="Heartbeat timeout">
            <Slider value={s.watchdog.heartbeat_timeout_s} min={10} max={120} step={5} format={(v) => `${v} s`} onChange={(v) => update((x) => void (x.watchdog.heartbeat_timeout_s = v))} />
          </Field>
        </Row>
      </Section>

      <Section title="Hotkeys">
        <HotkeyList />
      </Section>

      <Section title="Defaults & JSON">
        <Row
          title="Export settings JSON"
          sub="Writes settings-export.json to the data folder. Copy it to src-tauri/defaults.json to make it the built-in defaults."
          right={
            <>
              <Button size="sm" onClick={exportJson} icon={<FileDown size={13} />}>
                Export
              </Button>
              <Button size="sm" kind="ghost" onClick={copyJson} icon={<Copy size={13} />} />
            </>
          }
        />
        <Row
          title="Import settings JSON"
          right={
            <Button size="sm" onClick={importJson} icon={<FolderOpen size={13} />}>
              Choose file
            </Button>
          }
        />
        <Row
          title="Reset to defaults"
          sub="Loads defaults.json that ships with the app."
          right={
            confirmReset ? (
              <>
                <Button
                  size="sm"
                  kind="danger"
                  onClick={async () => {
                    setSettings(await api.settingsReset());
                    setConfirmReset(false);
                  }}
                  icon={<RotateCcw size={13} />}
                >
                  Reset everything
                </Button>
                <Button size="sm" kind="ghost" onClick={() => setConfirmReset(false)}>
                  Cancel
                </Button>
              </>
            ) : (
              <Button size="sm" kind="danger" onClick={() => setConfirmReset(true)} icon={<RotateCcw size={13} />}>
                Reset
              </Button>
            )
          }
        />
        {msg && (
          <div className="px-4 py-2 border-b border-line">
            <Pill tone="mute">
              <span className="truncate max-w-[340px] select-text">{msg}</span>
            </Pill>
          </div>
        )}
      </Section>

      <Section title="Presets">
        <Row title="Save current" sub="Snapshot every setting under a name, e.g. one per server.">
          <div className="flex gap-2">
            <TextField value={presetName} onChange={setPresetName} placeholder="e.g. halloween main" />
            <Button onClick={savePreset} disabled={!presetName.trim()} icon={<Save size={13} />}>
              Save
            </Button>
          </div>
        </Row>
        {presets.map((p) => (
          <Row
            key={p}
            title={p}
            right={
              <>
                <Button size="sm" onClick={async () => setSettings(await api.presetLoad(p))} icon={<Upload size={13} />}>
                  Load
                </Button>
                <Button
                  size="sm"
                  kind="ghost"
                  onClick={async () => {
                    await api.presetDelete(p);
                    setPresets(await api.presetList());
                  }}
                  icon={<Trash2 size={13} />}
                />
              </>
            }
          />
        ))}
      </Section>

      <Section title="App">
        <Row title="Version" sub={version} />
        <Row
          title="Data folder"
          sub={<span className="font-mono text-[11px] select-text">{dataDir}</span>}
          right={<Button size="sm" kind="ghost" onClick={() => api.openPath(dataDir)} icon={<FolderOpen size={13} />} />}
        />
      </Section>
    </div>
  );
}

function Field({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="flex items-center min-h-10 py-1">
      <div className="text-fg-dim w-32 shrink-0">{label}</div>
      <div className="ml-auto">{children}</div>
    </div>
  );
}
