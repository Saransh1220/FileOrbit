import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./theme.css";
import { Ring } from "./ring/Ring";
import { ManagerDemo } from "./manager/ManagerDemo";
import { RingDemo } from "./ring/RingDemo";
import { Toast } from "./toast/Toast";
import { Workspace } from "./workspace/Workspace";
import { WorkspaceDemo } from "./workspace/WorkspaceDemo";

// Each secondary window loads index.html#<name>; every other window is the app.
const views: Record<string, () => React.JSX.Element | null> = {
  "#overlay": Ring,
  "#toast": Toast,
  "#workspace": Workspace,
};
if (import.meta.env.DEV) {
  views["#ring-demo"] = RingDemo;
  views["#manager-demo"] = ManagerDemo;
  views["#workspace-demo"] = WorkspaceDemo;
}
const View = views[window.location.hash] ?? App;

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode><View /></React.StrictMode>,
);
