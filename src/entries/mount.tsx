import { type ReactNode, StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { bridge, isTauri } from "../bridge";
import "../styles/tokens.css";
import "../styles/base.css";

interface MountOptions {
  /** The taskbar strip follows the Windows mode instead of the app mode. */
  followTaskbarTheme?: boolean;
}

/** Mounts a window's UI and marks the host, so styles can tell the app from the dev browser. */
export function mount(ui: ReactNode, { followTaskbarTheme = false }: MountOptions = {}) {
  const root = document.documentElement;
  root.dataset.host = isTauri ? "tauri" : "browser";

  const forcedTheme = new URLSearchParams(window.location.search).get("theme");
  if (forcedTheme === "light" || forcedTheme === "dark") root.dataset.theme = forcedTheme;

  bridge.hostInfo().then((host) => {
    root.dataset.material = host.material;
    if (followTaskbarTheme && !forcedTheme) root.dataset.theme = host.taskbarTheme;
  });
  if (followTaskbarTheme && !forcedTheme) {
    bridge.onTaskbarTheme((theme) => {
      root.dataset.theme = theme;
    });
  }

  const container = document.getElementById("root");
  if (!container) throw new Error("Missing #root element");
  const reactRoot = createRoot(container);
  reactRoot.render(<StrictMode>{ui}</StrictMode>);
  // A hot update that reaches the entry runs it again; the old root must go first.
  import.meta.hot?.dispose(() => reactRoot.unmount());
}
