import { CompressPanel } from "./CompressPanel";
import { FileList } from "./FileList";
import { FormatPanel } from "./FormatPanel";
import { PresetBar } from "./PresetBar";
import { PreviewPane } from "./PreviewPane";
import { ResizePanel } from "./ResizePanel";
import { Section } from "./Section";
import { ActionBar, type SaveStatus } from "./ActionBar";
import { PRESETS } from "./presets";
import type { Settings } from "./settings";
import type { Estimate, ImageFile } from "./types";
import "../manager/skins/tokens.css";
import "./ImageToolView.css";

export interface ImageToolViewProps {
  files: ImageFile[];
  selected: number;
  settings: Settings;
  estimate: Estimate | null;
  resultUrl: string | null;
  problems: string[];
  canSave: boolean;
  status: SaveStatus;
  onSelect: (i: number) => void;
  onRemove: (i: number) => void;
  onAdd: () => void;
  onSettings: (next: Settings) => void;
  onSave: () => void;
  onShowSaved: (path: string) => void;
}

/**
 * The picture tool: the list on the left, before and after in the middle, the three parts
 * of the job on the right. State comes from props so every state can be previewed without Tauri.
 */
export function ImageToolView(props: ImageToolViewProps) {
  const { settings, onSettings } = props;
  const patchResize = (patch: Partial<Settings["resize"]>) => onSettings({ ...settings, resize: { ...settings.resize, ...patch } });
  const patchCompress = (patch: Partial<Settings["compress"]>) => onSettings({ ...settings, compress: { ...settings.compress, ...patch } });
  const pickPreset = (id: string) => {
    const preset = PRESETS.find((p) => p.id === id);
    if (preset) onSettings(preset.apply(settings));
  };

  return (
    <div className="it">
      <FileList files={props.files} selected={props.selected} onSelect={props.onSelect} onRemove={props.onRemove} onAdd={props.onAdd} />
      <main>
        <PreviewPane file={props.files[props.selected] ?? null} estimate={props.estimate} resultUrl={props.resultUrl} />
      </main>
      <aside className="settings" aria-label="Settings">
        <PresetBar onPick={pickPreset} />
        <Section title="Resize" hint="Change the dimensions" on={settings.resize.on} onToggle={(on) => patchResize({ on })}>
          <ResizePanel value={settings.resize} onChange={patchResize} />
        </Section>
        <Section title="Compress" hint="Make the file smaller" on={settings.compress.on} onToggle={(on) => patchCompress({ on })}>
          <CompressPanel value={settings.compress} format={settings.format} onChange={patchCompress} />
        </Section>
        <Section title="Format" hint="Output type" on>
          <FormatPanel value={settings.format} onChange={(format) => onSettings({ ...settings, format })} />
        </Section>
      </aside>
      <ActionBar count={props.files.length} problems={props.problems} canSave={props.canSave} status={props.status} onSave={props.onSave} onShowSaved={props.onShowSaved} />
    </div>
  );
}
