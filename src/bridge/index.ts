import { createBrowserBridge } from "./browser";
import { tauriBridge } from "./tauri";
import type { Bridge } from "./types";

export type { Bridge, NotifyLevels, RefreshMode, Settings, Unsubscribe } from "./types";

export const isTauri = "__TAURI_INTERNALS__" in window;

export const bridge: Bridge = isTauri ? tauriBridge : createBrowserBridge();

/** The service the flyout opens on until the user picks another tab. */
export const PRIMARY_PROVIDER = "claude";
