# Architecture

Limvo is a Tauri 2 application: a Rust core that owns everything touching
the system, and a React interface that only draws what the core reports.

```
          ┌──────────────── Rust core (src-tauri/src) ────────────────┐
          │                                                            │
 sign-in ─┤ providers/ claude codex cursor antigravity (one per service)│
 files    │        │                  │                                │
          │        ▼                  ▼                                │
 ~/.claude│ activity.rs ──nudge──▶ poller.rs ── Vec<ProviderReport> ─┐ │
 ~/.codex │ (file events)     one slot per service: schedule,      │ │
          │                   back-off, last good snapshot         │ │
          │                                                      │     │
          │ settings.rs  tray.rs  flyout.rs  card.rs  taskbar.rs   │ │
          └────────────────────────────────────────────────────────┼─┘
                                                     events      │
                         ┌───────────────────────────────────────┘
                         ▼
          ┌──────────── Interface (src) ──────────────┐
          │ flyout window   strip window   card window │
          │ core/ pace, alerts, pins  i18n/ 11 langs   │
          └────────────────────────────────────────────┘
```

## The data model

`UsageSnapshot` (`src/core/snapshot.ts`, mirrored in
`src-tauri/src/snapshot.rs`) is provider-neutral, so Claude and Codex share
every screen:

- `headline`: the window that matters most, drawn large.
- `windows`: every other limit, such as a model cap or the five-hour session.
- `breakdown`: how the headline window splits across products.
- `plan`: the plan badge (`max_20x`, `chatgpt_plus`, ...), which also picks the
  plan colour in `tokens.css`.
- `extraUsage`: paid credits spent this period, when the service reports them.

`ProviderReport` wraps it with the provider's id and name and a state (`ok`, `offline`, `auth_required`,
`no_credentials`, `error`, `loading`). On failure the last good snapshot is
kept, so the interface can show it as stale instead of going blank.

Both sides are tested against `fixtures/report.sample.json`: the Rust test
round-trips it through serde, the TypeScript test checks that the field sets
match exactly. Renaming a field on one side fails a test.

## Services

Each service implements the `UsageProvider` trait in
`src-tauri/src/providers` (see [CONTRIBUTING](../CONTRIBUTING.md#add-a-service))
and is listed in `providers::all()`. The host always reports every provider;
the interface hides the ones without a sign-in (`core/pins.ts`,
`visibleReports`), so signing in to a service makes its tab appear without a
restart.

| Service     | Sign-in it reads                                               | Endpoint                             | Activity folder                       |
| ----------- | -------------------------------------------------------------- | ------------------------------------ | ------------------------------------- |
| Claude      | Claude desktop app (DPAPI), then `~/.claude/.credentials.json` | `api.anthropic.com/api/oauth/usage`  | `~/.claude/projects`                  |
| Codex       | `~/.codex/auth.json` (ChatGPT sign-in)                         | `chatgpt.com/backend-api/wham/usage` | `~/.codex/sessions`                   |
| Cursor      | `state.vscdb` of the editor, SQLite opened read-only           | `api2.cursor.sh` dashboard service   | Cursor `workspaceStorage`             |
| Antigravity | Credential Manager entry `gemini:antigravity`                  | `cloudcode-pa.googleapis.com` quotas | `~/.gemini/antigravity/conversations` |

A service whose sign-in lives in a folder (Claude Code, Codex) can have
extra accounts (`accounts.rs`): each is the same provider bound to another
folder, with the report id `<service>~<n>` and `serviceId` naming the service
for its mark. The folder is chosen in the system dialog the core opens
(`add_account`); the interface never sends a path. Adding or removing one
hands the poller a new provider list (`PollerHandle::set_providers`), which
keeps the schedule and last reading of every provider that stays.

Windows come in four kinds: `session` (five hours), `weekly`, `monthly`
(a billing cycle, Cursor) and `scoped` (a weekly cap of one model, named by
its label: Claude model caps and Antigravity's per-model quotas). The kind
picks the label and the pace sentence.

## Refreshing

One task in `poller.rs` owns all network access and fetches the services one
after another, so there is never more than one request in flight. Each service
has its own slot with its own due time, failure count and last good snapshot,
so one service failing never delays or blanks another. Everything else only
wakes the task:

| Trigger                        | Effect                                                                  |
| ------------------------------ | ----------------------------------------------------------------------- |
| Timer                          | Adaptive: 1 minute while active, 5 minutes idle. Or 2, 5 or 15 minutes. |
| A service's tool writes a file | Wakes the poller if the last reading is older than a minute.            |
| Flyout opens, _Refresh_        | Wakes the poller; within 10 seconds of the last reading it reuses it.   |
| Failure                        | Back-off from 30 seconds to 15 minutes; `Retry-After` is honoured.      |
| Signed out                     | Checks again every 2 minutes, since signing in happens elsewhere.       |

## Windows

All windows are declared in `tauri.conf.json` and created hidden at start-up;
they are shown, moved and hidden, never recreated.

- **flyout**: Mica, no frame, always on top. Opens above whatever opened it
  (tray icon, strip, card), clamped to the work area, and hides on blur like
  system flyouts. The interface reports its content height so the window hugs
  it.
- **strip**: a transparent tool window kept over the taskbar, left of the
  notification area. Every other monitor's taskbar (`Shell_SecondaryTrayWnd`)
  gets a copy, `strip-1`, `strip-2` and so on, built at run time from the
  same declaration; clicking one opens the flyout above it. A secondary
  taskbar draws its clock in XAML with no window to measure, so the copy
  keeps the Windows 11 clock width free. A strip that was minimised or hidden
  (Win+M, screen tools) is shown again on the next tick. It is not re-parented into `Shell_TrayWnd`, because a
  WebView2 child of the taskbar loses its transparency. Instead a watcher
  thread re-asserts its position and z-order every 400 ms and hides it for an
  auto-hidden taskbar or a full-screen window on the same monitor.
- **card**: a desktop gadget. It shows without taking focus, sinks to the
  bottom of the z-order when released, and saves its position when the drag
  ends.

Hidden WebView2 windows throttle their timers, so entrance animations are
started by a `flyout-shown` event from the core rather than by React mounting.

## Interface

- `src/core` is pure TypeScript with no React: pace projection, the budget
  to the reset, alert rules, the week history and taskbar pins, unit tested.
  The budget uses the projection's elapsed floor, so after the first tenth of
  the window its rate so far is over the budget exactly when the projection
  is over 100%. The budget line takes its warning colour from the pace label,
  so the two always agree.
- The flyout shows one tab per visible service (`ui/flyout/ProviderTabs.tsx`)
  and remembers the chosen tab. Plan colours are set per element with
  `data-plan`, so every tab, pin and taskbar reading wears its own service's
  colour.
- **Pins** are stored in the settings as `<provider>:<window>` keys, at most
  six, in the order they were turned on. `null` means each service's main
  limit. Pins of a signed-out service are skipped, and when none is left the
  taskbar falls back to the main limits instead of going empty.
- `src/i18n` holds the message catalogues. Only words are translated; numbers,
  dates and durations come from `Intl` for the active locale.
- `src/bridge` is the only place that talks to the host. `tauri.ts` calls the
  core; `browser.ts` serves scenarios so the interface runs in a plain browser.
- The tray icon and its menu are drawn by the core, which has no translations,
  so the flyout window (alive for the whole session) sends the translated texts
  down whenever the language or the reading changes. The tooltip lists the
  pinned limits, cut by whole lines to the 127 characters Windows allows.
- Provider marks come from `public/brand/<id>.svg` (and `<id>-dark.svg` for
  dark surfaces) on the user's machine; without them the Limvo mark is drawn.

## Notifications

The flyout window evaluates each new report with `core/alerts.ts`. What has
been announced for each window is remembered in `localStorage`, keyed by
`<provider>:<window>` and reset time, so a restart never repeats a notification and turning
notifications back on never replays old news.

Only the main limit gets the forecast warning: once per period, when the
projection reaches 105% after the first tenth of the period has passed, and
only until a level such as 80% has been announced, which then carries the
news. Its body is the same budget the flyout shows.
