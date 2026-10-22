import { Page } from "../parts/Page";
import { STATUS_HELP, STATUS_LABEL, STATUS_TONE, WINDOWS_ADMIN_NOTE } from "../hookStatus";
import { SECTIONS } from "../sections";
import type { HookStatus } from "../types";
import "./ShortcutPage.css";

const BLURB = SECTIONS.find((s) => s.id === "shortcut")?.blurb;

/** Whether this window is running on Windows (decides the file manager's name and the admin note). */
const onWindows = () => navigator.userAgent.includes("Windows");

/** Whether Shift+drag is working, what to do if not, and how to use it. */
export function ShortcutPage({ status }: { status: HookStatus }) {
  const help = STATUS_HELP[status];
  const manager = onWindows() ? "Explorer" : "Finder";
  const steps = [
    `Hold Shift and drag a file out of ${manager} or from the desktop.`,
    "A ring of formats opens around the pointer.",
    "Drop the file on a format. The converted file appears next to the original.",
  ];

  return (
    <Page title="Shift+drag" blurb={BLURB}>
      <div className={`status-card ${STATUS_TONE[status]}`}>
        <span className="status-dot" aria-hidden />
        <div>
          <h2>{STATUS_LABEL[status]}</h2>
          {help && <p>{help}</p>}
        </div>
      </div>
      {onWindows() && status === "listening" && <p className="admin-note">{WINDOWS_ADMIN_NOTE}</p>}

      <h2 className="eyebrow">How it works</h2>
      <ol className="steps">
        {steps.map((step, index) => (
          <li key={step}>
            <span className="step-number">{index + 1}</span>
            {step}
          </li>
        ))}
      </ol>
    </Page>
  );
}
