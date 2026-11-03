<div align="center">

<img src="src-tauri/icons/128x128.png" width="72" alt="" />

# Headroom

**Your Claude plan limits and the pace you are using them at, right in the Windows taskbar.**

[Download](#install) · [Features](#features) · [How it works](#how-it-works) · [Setups](#setups-and-scenarios) · [Build from source](#build-from-source) · [Contributing](CONTRIBUTING.md)

</div>

---

Headroom puts the weekly limit of your Claude plan next to the clock, tells you
whether you are on track to make it to the reset, and stays out of the way
until you want the details. It reads the same numbers as the usage page on
claude.ai, refreshes them on its own while you work, and nudges you with a
Windows notification before you run out.

<p align="center">
  <img src="docs/images/flyout-light.png" width="380" alt="The Headroom flyout in the light theme: 51% of the weekly limit used, resets in 1 day 16 hr, on track with a projection of 67%" />
  <img src="docs/images/flyout-dark.png" width="380" alt="The same flyout in the dark theme" />
</p>

<p align="center">
  <img src="docs/images/strip-dark.png" height="64" alt="The taskbar reading in the dark taskbar: 51% until Tue, with a slim meter and the pace tick" />
  &nbsp;
  <img src="docs/images/strip-light.png" height="64" alt="The taskbar reading in the light taskbar" />
</p>

## Features

- **Taskbar reading.** `51% until Tue` with a slim meter, docked next to the
  notification area. Hover lights it up like any taskbar button; click it to
  open the flyout.
- **A counter with some craft.** The percentage rolls in on drum wheels like an
  old odometer: each digit drops in from above, and a wheel only moves on when
  the one to its right passes nine. The tray icon is alive too: the crossbar of
  its H rises with your usage, so the icon itself shows how much headroom is
  left.

  <img src="docs/images/counter-frames.png" width="640" alt="Four frames of the counter rolling up from 26% to 51%" />

- **Your plan at a glance.** Headroom reads which plan the sign-in belongs to
  and shows it as a badge: Free, Pro, Max 5×, Max 20×, Team and Enterprise.
  With _Plan colours_ on, the whole widget takes the plan's accent: graphite,
  terracotta, copper, rose, teal or indigo. Status colours keep their meaning.
- **A forecast, not just a reading.** A small chart plots this week against an
  even pace, with today's rate carried on to the reset. When that rate would
  empty the limit early, Headroom says when.

  <img src="docs/images/flyout-forecast.png" width="380" alt="The flyout running hot at 78%, with the week chart, the forecast that the limit runs out on Monday, and extra usage of 12.40 dollars" />

- **Extra usage.** When paid credits are turned on, what you spent this month
  shows next to the other limits, in your currency.
- **Keyboard shortcut.** Win+Shift+H opens the flyout from anywhere, or
  Ctrl+Alt+H when another app holds the first.
- **Pace, not just a percentage.** The tick on the meter marks how much of the
  week has passed. Headroom projects your usage to the reset and labels it
  _On track_, _Tight_, _Running hot_ or _Limit reached_.
- **The whole picture in one flyout.** Weekly limit, time to reset, the split
  between Claude Code, chats and Cowork, and model-specific caps such as
  _Fable · weekly_.
- **Live without polling all day.** While you work in Claude Code the reading
  updates about once a minute; when you are idle it slows down to every five
  minutes. Fixed intervals are one click away.
- **Windows notifications** at 80% and 95%, when a limit is reached and when a
  busy week resets.
- **Desktop card.** Pin a compact card to the desktop, drag it anywhere, and it
  stays below your windows.

  <img src="docs/images/card-dark.png" width="300" alt="The desktop card in the dark theme" />
- **Native feel.** Mica material, Windows 11 motion and controls, light and
  dark themes (the strip follows the taskbar theme, the flyout follows the app
  theme), respects _Animation effects_ and high DPI.
- **Eleven languages**, including right-to-left Hebrew: English, Polski,
  Deutsch, Español, Français, Italiano, Português (Brasil), Українська, עברית,
  日本語 and 简体中文. Follows the Windows display language or your choice.

  <img src="docs/images/flyout-polish-hot.png" width="300" alt="The flyout in Polish, dark theme, running hot at 78%" />
  <img src="docs/images/flyout-hebrew.png" width="300" alt="The flyout in Hebrew, laid out right to left" />
- **Small and private.** A few megabytes, no telemetry, no account, no server
  of its own. See [Privacy and security](#privacy-and-security).

## Install

1. Download `Headroom_<version>_x64-setup.exe` from the
   [latest release](../../releases/latest).
2. Run it. It installs for your user only, so no administrator prompt.
   Choose your language on the first screen.
3. Headroom starts in the notification area and docks its reading in the
   taskbar. Make sure you are signed in to the Claude desktop app or Claude
   Code (see [Setups](#setups-and-scenarios)).
4. Windows tucks new tray icons behind the `^` arrow. To keep the H next to the
   clock, drag it out of the overflow, or turn it on under _Settings >
   Personalization > Taskbar > Other system tray icons_.

<p align="center">
  <img src="docs/images/installer-welcome.png" width="420" alt="The Headroom installer welcome page with the language already matched to Windows" />
</p>

> **"Windows protected your PC"**
> Until releases are code-signed, Microsoft SmartScreen shows this warning for
> new downloads. Choose **More info**, check that the publisher and file name
> match the release, then **Run anyway**. You can verify the download with the
> SHA-256 checksum published next to each release.

**Updating:** download the newest installer and run it. It upgrades in place
and keeps your settings.

**Requirements:** Windows 10 or Windows 11, 64-bit. The WebView2 runtime ships
with Windows 11; on Windows 10 the installer fetches it if it is missing.

**Uninstall:** _Settings > Apps > Installed apps > Headroom > Uninstall_. Your
preferences live in `%APPDATA%\app.headroom.desktop` and can be deleted
afterwards.

## Using Headroom

| Where           | What it does                                                                                                                                                       |
| --------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Taskbar reading | Click to open or close the flyout.                                                                                                                                 |
| Tray icon       | Left click opens the flyout; right click offers _Refresh_ and _Quit_.                                                                                              |
| Flyout          | Shows the limits. _Open usage page_ goes to claude.ai, _Refresh_ reads now, the language code switches language, the sliders icon opens settings. `Esc` closes it. |
| Desktop card    | Drag it to move it; double click opens the flyout.                                                                                                                 |

### Settings

| Setting            | Default  | Notes                                                                                             |
| ------------------ | -------- | ------------------------------------------------------------------------------------------------- |
| Language           | System   | Any of the eleven languages, or follow Windows.                                                   |
| Start with Windows | Off      | Adds Headroom to your user's startup apps.                                                        |
| Show in taskbar    | On       | Turn off to keep only the tray icon.                                                              |
| Pin to desktop     | Off      | Shows the desktop card. Its position is remembered.                                               |
| Notifications      | On       | 80%, 95%, limit reached, and the reset after a week above 80%. Each one is shown once per window. |
| Update frequency   | Adaptive | Adaptive follows Claude Code activity. 2, 5 or 15 minutes are fixed schedules.                    |

## How it works

Headroom asks the same endpoint the Claude apps use to draw their usage
screens, `api.anthropic.com/api/oauth/usage`, with the sign-in that is already
on your computer. It does not have its own login.

It looks for a sign-in in this order and uses the first one that works:

1. **Claude desktop app.** The app keeps its sign-in in its `config.json`,
   encrypted with Windows DPAPI for your user account. Headroom decrypts it in
   memory, read-only, to borrow the access token. Both the classic installer
   (`%APPDATA%\Claude`) and the Microsoft Store version
   (`%LOCALAPPDATA%\Claude-Data`) are supported.
2. **Claude Code CLI.** `~/.claude/.credentials.json`, or
   `%CLAUDE_CONFIG_DIR%\.credentials.json` when that variable is set.

Headroom never refreshes, writes or copies a token. An expired sign-in is
reported as _sign in again_ instead of being renewed behind your back, because
refreshing would sign Claude Code out.

**Adaptive updates.** The usage endpoint cannot push changes, so Headroom
watches the folder where Claude Code writes its transcripts
(`~/.claude/projects`). It only notices that files changed; it never opens
them. A change means you are working, so the next reading comes within a
minute. Manual refreshes closer than ten seconds apart reuse the last answer,
and rate limits from the server are honoured.

> **This endpoint is undocumented.** Anthropic can change it at any time.
> Headroom tolerates missing and new fields, keeps showing the last good
> reading when a refresh fails, and says so in the flyout, but a larger change
> on the server side may need an update of the app.

## Setups and scenarios

| Your setup                              | What happens                                                                                                                |
| --------------------------------------- | --------------------------------------------------------------------------------------------------------------------------- |
| Claude desktop app, signed in           | Works out of the box, including the Microsoft Store version.                                                                |
| Claude Code CLI only                    | Works if you have signed in with `claude` at least once. Run `claude` and `/login` if the flyout says your sign-in expired. |
| Both installed                          | Headroom picks the freshest sign-in that has the required scope.                                                            |
| Signed out everywhere                   | The flyout asks you to sign in. Headroom checks again every two minutes, no restart needed.                                 |
| Custom Claude Code folder               | Set `CLAUDE_CONFIG_DIR` before starting Headroom; it follows the same variable as Claude Code.                              |
| Several monitors                        | The reading docks on the main taskbar. Flyout and card open on the monitor you use them from.                               |
| Auto-hide taskbar                       | The reading hides with the taskbar and returns with it.                                                                     |
| Full-screen game, video or presentation | The reading steps aside on that monitor only. Live wallpapers and full-screen windows on other monitors do not hide it.     |
| Vertical taskbar (Windows 10)           | The taskbar reading is not shown; tray icon, flyout and card work.                                                          |
| Windows 10                              | No Mica, so the flyout uses a solid surface. Everything else is the same.                                                   |
| No usage yet this week                  | The per-product section stays hidden until something is used.                                                               |

## Privacy and security

- No telemetry, analytics or crash reporting. The only network request is the
  usage request to `api.anthropic.com`.
- Tokens stay in memory for the duration of one request and are never logged.
  Logs record only which source was used and HTTP status codes.
- Everything that touches credentials, the network and the system lives in the
  Rust core (`src-tauri/src`). The interface runs with a strict content
  security policy and can only call the commands listed in
  [`src-tauri/src/commands.rs`](src-tauri/src/commands.rs).
- Logs: `%LOCALAPPDATA%\app.headroom.desktop\logs`. Settings:
  `%APPDATA%\app.headroom.desktop\settings.json`.

Found a security problem? Please follow [SECURITY.md](SECURITY.md) instead of
opening a public issue.

## Troubleshooting

| Symptom                                        | Try this                                                                                                |
| ---------------------------------------------- | ------------------------------------------------------------------------------------------------------- |
| _Sign in to the Claude app to see your limits_ | Sign in to the Claude desktop app, or run `claude` once in a terminal.                                  |
| _Your Claude sign-in expired_                  | Open the Claude desktop app or run `claude`; both renew the sign-in themselves.                         |
| _Offline. Last reading ..._                    | No connection to `api.anthropic.com`, or the server asked to slow down. Headroom retries with back-off. |
| The taskbar reading is missing                 | Check _Settings > Show in taskbar_, a full-screen window on the main monitor, or a vertical taskbar.    |
| Numbers look stale                             | _Refresh_ in the flyout. Adaptive mode slows down after five idle minutes.                              |
| Something else                                 | Attach the newest file from the logs folder to an issue. It contains no tokens.                         |

## Build from source

You need:

- [Node.js](https://nodejs.org) 22 or later
- [Rust](https://rustup.rs) (stable, 1.95 or later)
- Visual Studio Build Tools with the _Desktop development with C++_ workload
- WebView2 (preinstalled on Windows 11)

```powershell
winget install Rustlang.Rustup
winget install Microsoft.VisualStudio.2022.BuildTools --override "--quiet --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
```

Then:

```powershell
npm install
npm run tauri dev      # the app with hot reload
npm run dev            # only the interface, in a browser, with sample data
npm run check          # types, lint and unit tests
cd src-tauri; cargo test; cargo clippy --all-targets
npm run tauri build    # installer in src-tauri/target/release/bundle/nsis
```

`npm run dev` serves the interface with built-in scenarios, which is the
quickest way to work on design: `?scenario=reference|fresh|tight|hot|reached|offline|auth|signedOut|error`,
`&locale=he`, `&theme=dark`, and `&now=<ISO time>` for reproducible screenshots.
Open `/strip.html` and `/card.html` for the other windows.

### Your own brand mark

Headroom ships with its own mark. To see a provider's official logo in the
app, put the SVG from the provider's brand or press page at
`public/brand/<provider>.svg` (for example `claude.svg`) and rebuild. Brand
files are ignored by git; please do not commit them to a public fork.

## Project layout

```
src/                 Interface (React 19, TypeScript)
  core/              Pure logic: pace, alerts, the shared data model
  i18n/              Translations and Intl formatting
  ui/                Flyout, taskbar strip, desktop card, controls
src-tauri/src/       Core (Rust, Tauri 2)
  providers/         One module per service; Claude today
  poller.rs          Schedule, back-off, adaptive refresh
  taskbar.rs         Docking next to the notification area
  flyout.rs card.rs  Window placement and behaviour
fixtures/            Shared contract fixtures for both sides
docs/                Architecture notes and images
```

Read [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for the design, and
[CONTRIBUTING.md](CONTRIBUTING.md) to add a language or another service.

## Roadmap

- macOS menu bar version
- In-app updates from GitHub Releases
- Readings on secondary taskbars
- Code-signed releases and `winget install Headroom`
- More providers through the same provider interface
- A Windows 11 Widgets board card

## Disclaimer

Headroom is an independent open-source project. It is not affiliated with,
endorsed by or sponsored by Anthropic. Claude is a trademark of Anthropic,
PBC, used here only to describe what the app works with.

## License

[MIT](LICENSE)
