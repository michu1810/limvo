// Renders the Microsoft Store screenshots (docs/store/scenes.html, 3840x2160)
// and logos (docs/store/art.html: 2160x2160 box art, 1440x2160 poster).
//
//   npm run dev                       (in another terminal)
//   node docs/store/render.mjs [chrome.exe] [1-6 | hero | box | poster | pl:1 ...]
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
const ART = { box: [1080, 1080], poster: [720, 1080] };
// "pl:3" renders scene 3 in Polish; the Polish files get a "-pl" suffix.
const targets = (args.length ? args : ["1", "2", "3", "4", "5", "6", "hero", "box", "poster"]).map(
  (target) => {
    const [lang, name] = target.includes(":") ? target.split(":") : ["en", target];
    const suffix = lang === "en" ? "" : `-${lang}`;
    return name in ART
      ? { url: `art.html?a=${name}`, size: ART[name], file: `limvo-store-${name}.png` }
      : {
          url: `scenes.html?s=${name}&lang=${lang}`,
          size: [1920, 1080],
          file: `limvo-store-${name}${suffix}.png`,
        };
  },
);
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

await send("Emulation.setEmulatedMedia", {
  features: [{ name: "prefers-reduced-motion", value: "reduce" }],
});

for (const { url, size, file } of targets) {
  await send("Emulation.setDeviceMetricsOverride", {
    width: size[0],
    height: size[1],
    deviceScaleFactor: 2,
    mobile: false,
  });
  await send("Page.navigate", { url: `http://localhost:1420/docs/store/${url}` });
  let ready = false;
  for (let i = 0; i < 150 && !ready; i++) {
    await sleep(200);
    const { result } = await send("Runtime.evaluate", { expression: "document.title" });
    ready = result.value === "ready";
  }
  if (!ready) throw new Error(`${url} never became ready`);
  // Let the compositor paint the last frame at the new size before capturing it.
  await sleep(600);
  const { data } = await send("Page.captureScreenshot", {
    format: "png",
    captureBeyondViewport: false,
  });
  writeFileSync(join(out, file), Buffer.from(data, "base64"));
  console.log(join(out, file));
}

socket.close();
browser.kill();
