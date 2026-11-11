# Microsoft Store listing

Texts for the Limvo submission in Partner Center. Screenshots (1920×1080) are
built from `docs/images`. Keep other services' names to describing what Limvo
works with: they are not in the product name, the logo or the search terms.

## Properties

- Category: **Productivity** (subcategory none)
- Privacy policy URL: `https://github.com/michu1810/limvo#privacy-and-security`
- Website: `https://github.com/michu1810/limvo`
- Support contact: `https://github.com/michu1810/limvo/issues`
- System requirements: Windows 10 version 2004 or later, x64. Minimum memory
  and other hardware: not required.

## Store listing: English (United States)

**Product name:** Limvo

**Short description** (shown on Windows 11 listing cards):

> Your AI plan limits in the Windows taskbar, with the pace you are using them at.

**Description:**

> Limvo puts the plan limits of the AI coding tools you use next to the clock. It shows how much of each allowance is gone, whether you are on track to make it to the reset, and when a limit will run out at today's pace.
>
> Limvo works with Claude (desktop app and Claude Code), Codex with a ChatGPT plan, Cursor, Google Antigravity and GitHub Copilot. A service appears as soon as you are signed in to it on this computer; there is nothing to configure and no account to create.
>
> • A reading in the taskbar: one limit as a sentence, or up to six limits side by side, on every monitor.
> • A forecast: Limvo projects your usage to the reset and labels it On track, Tight, Running hot or Limit reached.
> • The whole picture per service: weekly and five-hour windows, model caps, monthly included usage and extra usage.
> • Windows notifications: an early warning when your pace would run out before the reset, then 80% and 95%, when a limit is reached and when a busy period resets.
> • Light and dark themes, Mica, high DPI, and eleven languages including right-to-left Hebrew.
>
> Private by design: no telemetry, no analytics, no server of its own. Limvo only sends the usage requests to the services you are signed in to, over HTTPS. Sign-ins stay where the services keep them. Limvo is open source under the MIT license.
>
> Limvo is an independent project and is not affiliated with, endorsed by or sponsored by Anthropic, OpenAI, Anysphere or Google.

**Features** (one per line):

> Plan limits of your AI coding tools in the Windows taskbar
> Forecast of when each limit runs out at today's pace
> Weekly, five-hour, monthly and per-model limits
> Up to six limits side by side, on every monitor
> Notifications before a limit runs out and when it resets
> Desktop card and a keyboard shortcut to open it from anywhere
> No telemetry, no account, no server of its own
> Eleven languages, light and dark themes

**Search terms** (up to 7):

> AI usage, plan limits, usage tracker, taskbar widget, rate limit, AI coding, quota

**Screenshot captions:**

1. Every AI plan limit in one place
2. A forecast, not just a reading
3. Right in the Windows taskbar
4. Pin the limits you care about
5. Eleven languages, Hebrew right to left

## Store listing: Polski

**Krótki opis:**

> Limity Twoich planów AI na pasku zadań Windows, razem z tempem, w jakim je zużywasz.

**Opis:**

> Limvo pokazuje limity planów narzędzi AI do programowania obok zegara. Widać, ile z każdego limitu już zużyłeś, czy zmieścisz się do resetu i kiedy limit się skończy przy dzisiejszym tempie.
>
> Limvo działa z Claude (aplikacja desktopowa i Claude Code), Codex z planem ChatGPT, Cursorem, Google Antigravity i GitHub Copilot. Usługa pojawia się, gdy tylko jesteś w niej zalogowany na tym komputerze: niczego nie trzeba konfigurować ani zakładać konta.
>
> • Odczyt na pasku zadań: jeden limit jako zdanie albo do sześciu limitów obok siebie, na każdym monitorze.
> • Prognoza: Limvo przewiduje zużycie do resetu i oznacza je jako W normie, Na styk, Za szybko albo Limit osiągnięty.
> • Pełny obraz każdej usługi: okna tygodniowe i pięciogodzinne, limity modeli, miesięczne zużycie i dodatkowe zużycie.
> • Powiadomienia Windows przy 80% i 95%, po osiągnięciu limitu i po jego odnowieniu.
> • Jasny i ciemny motyw, Mica, wysokie DPI i jedenaście języków, w tym hebrajski pisany od prawej do lewej.
>
> Prywatność od początku: bez telemetrii, bez analityki, bez własnego serwera. Limvo wysyła tylko zapytania o zużycie do usług, w których jesteś zalogowany, przez HTTPS. Limvo jest open source na licencji MIT.
>
> Limvo to niezależny projekt, niezwiązany z Anthropic, OpenAI, Anysphere ani Google i przez nie niesponsorowany.

## Submission options

**Restricted capability `runFullTrust`, why it is needed:**

> Limvo is a desktop (Win32) app packaged with the Desktop Bridge. It runs as a full-trust process to draw its reading over the Windows taskbar on every monitor, to show a tray icon, to register a global keyboard shortcut, and to read the local sign-in files of the AI tools the user has installed (for example %USERPROFILE%\.claude and %APPDATA%\Cursor) in order to request the user's own usage from those services.

**Notes for certification:**

> Limvo shows the plan limits of AI coding tools the user is already signed in to on the same PC: the Claude desktop app or Claude Code, the Codex CLI or app, Cursor, Google Antigravity and the GitHub CLI for Copilot. It reads their existing local sign-in only to request that user's own usage from each service over HTTPS. Tokens are kept in memory for one request and are never stored, logged or sent anywhere else. There is no telemetry and no server of its own; see https://github.com/michu1810/limvo#privacy-and-security.
>
> On a test machine without any of these tools signed in, Limvo starts in the notification area and its flyout explains which services it supports and how to sign in. To see readings, install and sign in to Claude Code or Cursor with any account; the free plans work.
>
> The Store build does not update itself: updates come from the Store. "Launch at login" uses the package's startup task (Settings > Apps > Startup).
