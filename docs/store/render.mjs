// Renders docs/store/scenes.html into Microsoft Store screenshots at 3840x2160.
//
//   npm run dev                       (in another terminal)
//   node docs/store/render.mjs [chrome.exe] [scene numbers...]
//
// Uses any Chromium through the DevTools protocol: it waits until a scene sets
// its title to "ready", with reduced motion so every counter shows its final
// value, then captures it at device scale 2.

import { spawn } from "node:child_process";
import { mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const args = process.argv.slice(2);
const chrome = args[0]?.endsWith(".exe") ? args.shift() : process.env.CHROME;
if (!chrome) throw new Error("pass the path of chrome.exe (or set CHROME)");
const scenes = args.length ? args.map(Number) : [1, 2, 3, 4, 5, 6];
const out = join(import.meta.dirname, "..", "..", "src-tauri", "target", "store-screenshots");
mkdirSync(out, { recursive: true });

const port = 9333;
const profile = mkdtempSync(join(tmpdir(), "limvo-store-"));
const browser = spawn(chrome, [
  "--headless=new",
  `--remote-debugging-port=${port}`,
  `--user-data-dir=${profile}`,
  "--hide-scrollbars",
  "--force-color-profile=srgb",
  "about:blank",
]);

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

async function target() {
  for (let i = 0; i < 50; i++) {
    try {
      const pages = await (await fetch(`http://127.0.0.1:${port}/json/list`)).json();
      const page = pages.find((p) => p.type === "page");
      if (page) return page.webSocketDebuggerUrl;
    } catch {}
    await sleep(200);
  }
  throw new Error("Chromium did not start");
}

const socket = new WebSocket(await target());
await new Promise((r) => socket.addEventListener("open", r, { once: true }));
let next = 0;
const pending = new Map();
socket.addEventListener("message", (event) => {
  const message = JSON.parse(event.data);
  if (message.id && pending.has(message.id)) {
    const { resolve, reject } = pending.get(message.id);
    pending.delete(message.id);
    message.error ? reject(new Error(message.error.message)) : resolve(message.result);
  }
});
const send = (method, params = {}) =>
  new Promise((resolve, reject) => {
    const id = ++next;
    pending.set(id, { resolve, reject });
    socket.send(JSON.stringify({ id, method, params }));
  });

await send("Emulation.setDeviceMetricsOverride", {
  width: 1920,
  height: 1080,
  deviceScaleFactor: 2,
  mobile: false,
});
await send("Emulation.setEmulatedMedia", {
  features: [{ name: "prefers-reduced-motion", value: "reduce" }],
});

for (const scene of scenes) {
  await send("Page.navigate", { url: `http://localhost:1420/docs/store/scenes.html?s=${scene}` });
  let ready = false;
  for (let i = 0; i < 150 && !ready; i++) {
    await sleep(200);
    const { result } = await send("Runtime.evaluate", { expression: "document.title" });
    ready = result.value === "ready";
  }
  if (!ready) throw new Error(`scene ${scene} never became ready`);
  const { data } = await send("Page.captureScreenshot", {
    format: "png",
    captureBeyondViewport: false,
  });
  const file = join(out, `limvo-store-${scene}.png`);
  writeFileSync(file, Buffer.from(data, "base64"));
  console.log(file);
}

socket.close();
browser.kill();
