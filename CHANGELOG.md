# Changelog

All notable changes to Limvo are listed here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow
[Semantic Versioning](https://semver.org/).

## [Unreleased]

### Fixed

- "No connection" no longer shows when a service only asked Limvo to slow
  down. The last reading stays on screen, and Limvo waits at least two
  minutes, longer after each refusal. Activity, an opened flyout or Refresh
  no longer cut that wait short, which used to keep Claude refusing for
  hours.

## [0.3.0] - 2026-11-04

### Added

- Extra accounts: up to four more Claude Code or Codex accounts, each signed
  in from its own folder, added in _Settings > Extra accounts_. Each gets its
  own tab and taskbar limits, with a letter beside the service's mark to tell
  them apart. An extra Claude account reads only its folder, never the
  desktop app.

### Changed

- The running version shows in the settings header, without scrolling, and
  in the tray icon's tooltip.

## [0.2.0] - 2026-11-04

### Added

- In-app updates: Limvo checks GitHub Releases twice a day and offers a newer
  version in the flyout. It installs only when you choose _Update_ and only if
  the update is signed with the release key. _Settings > Check for updates_
  turns the check off. _Later_ hides a version for three days; a newer one
  shows at once.

### Fixed

- The taskbar reading no longer disappears after you open the hidden tray
  icons or click the taskbar: it comes back on top of the taskbar at once,
  below the tray overflow and menus.
- The taskbar reading no longer blinks off during Alt+Tab, Task View and
  window previews.
- A minimised taskbar reading (for example after Win+D) returns within a
  moment instead of up to half a second later.

## [0.1.1] - 2026-11-04

### Fixed

- The Claude plan is read from the account profile, so an upgrade shows at
  once with the plan's own name (for example Ultra) instead of the plan the
  sign-in had when it was created.

## [0.1.0] - 2026-11-04

### Added

- Four services: Claude, Codex (ChatGPT plans), Cursor and Google Antigravity,
  each with its logo, plan badge and colour, in tabs in the flyout.
- Cursor's monthly included usage, split into Auto and API.
- Antigravity's weekly quota per agent model.
- Taskbar pins: choose up to six limits across services; several limits sit
  side by side in the taskbar.
- Notifications, the week chart and the tray tooltip cover every service.
- A taskbar reading on every monitor, with the flyout opening above the one
  that was clicked.
- Long views scroll inside the flyout, which never grows past the screen.
- Taskbar reading next to the notification area, following the taskbar theme.
- Flyout with the weekly limit, time to reset, pace and projection, the split
  by product and model-specific caps.
- Adaptive refresh that follows activity in each service's tools, plus fixed
  schedules.
- Windows notifications at 80%, 95%, when a limit is reached and when a busy
  week resets.
- Desktop card that stays below other windows and remembers its position.
- Settings: language, start with Windows, taskbar reading, desktop card,
  notifications, update frequency.
- Sign-in from the Claude desktop app (classic and Microsoft Store) and the
  Claude Code CLI.
- Eleven languages, including right-to-left Hebrew.
- Rolling drum counter for the percentage, with odometer-style carries.
- Plan badge (Free, Pro, Max 5×, Max 20×, Team, Enterprise), styled per plan.
- Tray icon whose meter fills with usage, or the provider's own mark when it is bundled.
- Plan colours: the accent follows the account's plan.
- Week chart against an even pace, and a forecast of when the limit runs out.
- Extra usage (paid credits) among the other limits.
- System-wide shortcut, Win+Shift+H with Ctrl+Alt+H as the fallback.
- App icon, and a branded per-user installer with a language selector and
  installer texts translated into all eleven languages.
