# Architecture

Headroom is a Tauri 2 application: a Rust core that owns everything touching
the system, and a React interface that only draws what the core reports.

```
          ┌──────────────── Rust core (src-tauri/src) ────────────────┐
          │                                                            │
 sign-in ─┤ providers/claude   credentials, API call, mapping          │
 files    │        │                                                   │
          │        ▼                                                   │
 ~/.claude│ activity.rs ──nudge──▶ poller.rs ── ProviderReport ──┐     │
 projects │ (file events)         schedule, back-off, last good  │     │
          │                                                      │     │
          │ settings.rs  tray.rs  flyout.rs  card.rs  taskbar.rs │     │
          └──────────────────────────────────────────────────────┼─────┘
                                                     events      │
                         ┌───────────────────────────────────────┘
                         ▼
          ┌──────────── Interface (src) ──────────────┐
          │ flyout window   strip window   card window │
          │ core/ pace, alerts     i18n/ 11 languages  │
          └────────────────────────────────────────────┘
```

## The data model

`UsageSnapshot` (`src/core/snapshot.ts`, mirrored in
`src-tauri/src/snapshot.rs`) is provider-neutral:

- `headline`: the window that matters most, drawn large.
- `windows`: every other limit, such as a model cap or the five-hour session.
- `breakdown`: how the headline window splits across products.

`ProviderReport` wraps it with a state (`ok`, `offline`, `auth_required`,
`no_credentials`, `error`, `loading`). On failure the last good snapshot is
kept, so the interface can show it as stale instead of going blank.

Both sides are tested against `fixtures/report.sample.json`: the Rust test
round-trips it through serde, the TypeScript test checks that the field sets
match exactly. Renaming a field on one side fails a test.

## Refreshing

One task in `poller.rs` owns all network access, so there is never more than
one request in flight. Everything else only wakes it:

| Trigger                         | Effect                                                                  |
| ------------------------------- | ----------------------------------------------------------------------- |
| Timer                           | Adaptive: 1 minute while active, 5 minutes idle. Or 2, 5 or 15 minutes. |
| Claude Code writes a transcript | Wakes the poller if the last reading is older than a minute.            |
| Flyout opens, _Refresh_         | Wakes the poller; within 10 seconds of the last reading it reuses it.   |
| Failure                         | Back-off from 30 seconds to 15 minutes; `Retry-After` is honoured.      |
| Signed out                      | Checks again every 2 minutes, since signing in happens elsewhere.       |

## Windows

All windows are declared in `tauri.conf.json` and created hidden at start-up;
they are shown, moved and hidden, never recreated.

- **flyout**: Mica, no frame, always on top. Opens above whatever opened it
  (tray icon, strip, card), clamped to the work area, and hides on blur like
  system flyouts. The interface reports its content height so the window hugs
  it.
- **strip**: a transparent tool window kept over the taskbar, left of the
  notification area. It is not re-parented into `Shell_TrayWnd`, because a
  WebView2 child of the taskbar loses its transparency. Instead a watcher
  thread re-asserts its position and z-order every 400 ms and hides it for an
  auto-hidden taskbar or a full-screen window on the same monitor.
- **card**: a desktop gadget. It shows without taking focus, sinks to the
  bottom of the z-order when released, and saves its position when the drag
  ends.

Hidden WebView2 windows throttle their timers, so entrance animations are
started by a `flyout-shown` event from the core rather than by React mounting.

## Interface

- `src/core` is pure TypeScript with no React: pace projection and alert
  rules, unit tested.
- `src/i18n` holds the message catalogues. Only words are translated; numbers,
  dates and durations come from `Intl` for the active locale.
- `src/bridge` is the only place that talks to the host. `tauri.ts` calls the
  core; `browser.ts` serves scenarios so the interface runs in a plain browser.
- The tray icon and its menu are drawn by the core, which has no translations,
  so the flyout window (alive for the whole session) sends the translated texts
  down whenever the language or the reading changes.

## Notifications

The flyout window evaluates each new report with `core/alerts.ts`. What has
been announced for each window is remembered in `localStorage`, keyed by window
id and reset time, so a restart never repeats a notification and turning
notifications back on never replays old news.
