import "../App.css";
import { type HookStatus, STATUS_TEXT } from "./hookStatus";
import "./StatusSkins.css";

/** Presentational status window: state comes from props so it can be previewed without Tauri. */
export function StatusView({ status }: { status: HookStatus }) {
  return (
    <main className="app">
      <h1>Radial Convert</h1>
      <p className={`status ${status}`}>{STATUS_TEXT[status]}</p>
      <p className="hint">Change the look from the tray icon menu, under Theme.</p>
    </main>
  );
}
