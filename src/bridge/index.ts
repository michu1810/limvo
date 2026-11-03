import { createBrowserBridge } from "./browser";
import { tauriBridge } from "./tauri";
import type { Bridge } from "./types";

export type { Bridge, RefreshMode, Settings, Unsubscribe } from "./types";

export const isTauri = "__TAURI_INTERNALS__" in window;

export const bridge: Bridge = isTauri ? tauriBridge : createBrowserBridge();
