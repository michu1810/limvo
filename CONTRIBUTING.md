# Contributing to Limvo

Thanks for helping. This guide covers the setup, the standards every change
meets, and the two most common contributions: a new language and a new
service.

## Setup

Follow [Build from source](README.md#build-from-source), then:

```powershell
npm install
npm run tauri dev
```

For interface work, `npm run dev` is faster: it runs the windows in a browser
with sample data. See the scenarios listed in the README.

## Before you open a pull request

```powershell
npm run check                        # TypeScript, Biome, unit tests
cd src-tauri
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

CI runs the same commands on Windows. A pull request should:

- Change one thing, with a description of what and why.
- Keep the interface free of system access: credentials, network, files and
  windows belong to the Rust core.
- Add or update tests next to the code (`*.test.ts`, `#[cfg(test)]` modules).
- Work in light and dark themes, with _Animation effects_ off, and in a
  right-to-left language (`?locale=he`).
- Use plain hyphens in text. En and em dashes are rejected by CI.

## Add a language

1. Copy `src/i18n/locales/en.ts` to `src/i18n/locales/<code>.ts`, where `<code>`
   is a BCP 47 tag such as `nl` or `pt-PT`.
2. Translate every string. Keep the `{placeholders}` exactly as they are; a
   test fails if a placeholder is missing or extra.
3. Set `nativeName` (the language in itself), `shortLabel` (two or three
   characters for the footer button) and `dir` (`rtl` for right-to-left).
4. Register it in `src/i18n/locales/index.ts`.
5. Check it with `npm run dev` and `?locale=<code>`. Long labels must fit the
   380 px flyout; prefer shorter wording over truncation.

Numbers, dates and durations are formatted by `Intl` and need no translation.

## Add a service

A service is a _provider_: it reads one service's usage and returns the
neutral `UsageSnapshot` the interface already knows how to draw.

1. Create `src-tauri/src/providers/<name>/mod.rs` with a type that implements
   `UsageProvider`:

   ```rust
   impl UsageProvider for MyProvider {
       // Stable id: used in pins (`<id>:<window>`), brand files and plan colours.
       fn id(&self) -> &'static str { "my_service" }
       fn name(&self) -> &'static str { "My Service" }

       fn fetch<'a>(&'a self, http: &'a reqwest::Client) -> FetchFuture<'a> {
           Box::pin(async move {
               // Find credentials, call the API, map the answer.
               Ok(UsageSnapshot { /* ... */ })
           })
       }

       // Optional: folders the service's own tools write to while in use.
       fn activity_paths(&self) -> Vec<PathBuf> { Vec::new() }
   }
   ```

2. Map errors to `ProviderError`. The poller turns them into the states the
   flyout explains: no credentials, sign in again, offline, rate limited.
3. Keep the mapping a pure function and test it against a fixture of a real,
   anonymised response, as `providers/claude/api.rs`, `providers/codex.rs`,
   `providers/cursor.rs` and `providers/antigravity.rs` do.
4. Add it to `providers::all()`. The flyout gets a tab for it as soon as it
   reports a sign-in, and its limits show up in the taskbar pins.
5. Optional: give its plans a colour in `src/styles/tokens.css`
   (`[data-plan^="<prefix>"]`), and put the service's official logo at
   `public/brand/<id>.svg` on your own machine.

Rules for credentials, without exception:

- Read only. Never write, refresh, move or copy a credential.
- Never log one, return one to the interface, or put one in an error message.
  Give token types a redacting `Debug` implementation.
- Send it only to the service it belongs to, over HTTPS.

## Design and motion

The interface follows the Windows 11 flyout: Segoe UI Variable, 8 px window
corners, subtle fills on hover, Mica where available. Motion is part of the
design, with three rules:

- It runs on change only. Nothing loops while the window is idle; the taskbar
  strip is on screen all day.
- One part moves, not the whole: a knob slides, a number counts, a light
  passes along a meter once.
- `prefers-reduced-motion` turns it off. It maps to Windows _Animation
  effects_.

Colours come from the tokens in `src/styles/tokens.css`; components never use
raw colours.

## Commit messages

Write them in English, in the imperative ("Add Dutch translation"), with a
body that explains why when it is not obvious.
