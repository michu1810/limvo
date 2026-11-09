# Microsoft Store

Limvo ships to the Microsoft Store as an MSIX package that carries the same
`limvo.exe` as the installer. The Store signs the package, so Store installs
get no SmartScreen warning whether or not the installer is code-signed.

## What differs in the Store build

The app finds out at start whether it runs from a package
(`src-tauri/src/package.rs`) and then:

- never checks GitHub for updates, and hides _Check for updates_: the Store
  delivers updates;
- starts at sign-in through the package's startup task (`LimvoStartup` in
  [`AppxManifest.xml`](../src-tauri/msix/AppxManifest.xml)) instead of the
  registry `Run` key, which a package cannot reach. If the user turned Limvo
  off in Task Manager > Startup apps, only they can turn it back on there.

Everything else is the same build. Settings stay in
`%APPDATA%\io.github.michu1810.limvo`.

## Building the package

```powershell
npx tauri build --no-bundle
./scripts/pack-msix.ps1 -IdentityName <Name> -Publisher "<CN=...>" -PublisherDisplayName "<name>"
```

The package lands in `src-tauri/target/msix`. Without parameters the script
uses a local test identity. To try the Store build without the Store, turn on
Developer Mode, quit the installed Limvo (both builds are one app to
single-instance) and register the layout folder:

```powershell
Add-AppxPackage -Register src-tauri/target/msix/layout/AppxManifest.xml
Get-AppxPackage Limvo.Local | Remove-AppxPackage   # remove it again
```

## Releases (maintainers)

1. In [Partner Center](https://partner.microsoft.com/dashboard), register as
   an individual developer (free, with ID verification) and reserve the name
   **Limvo** under _Apps and games > New product > MSIX or PWA app_.
2. Copy the three values from _Product management > Product identity_ into
   the `release` environment as variables: `MSSTORE_IDENTITY_NAME`
   (Package/Identity/Name), `MSSTORE_PUBLISHER` (Package/Identity/Publisher,
   starts with `CN=`) and `MSSTORE_PUBLISHER_DISPLAY_NAME`.
3. Push a tag. The Release workflow keeps the package as the `limvo-msix`
   artifact of the run. For the very first release, download it and upload
   the `.msix` in a new Store submission by hand: the Store API can only
   update a product that is already live.

### Automatic Store updates

Once Limvo is live in the Store, every approved tag also goes to the Store
by itself: the `store` job of the Release workflow submits the same package
with the [Microsoft Store Developer CLI](https://learn.microsoft.com/windows/apps/publish/msstore-dev-cli/github-actions),
keeps the live listing and commits the submission for certification. Set it
up once:

1. In Partner Center, _Account settings > Tenants_: create a new Microsoft
   Entra ID tenant (free), or associate one you already have.
2. _Account settings > User management > Microsoft Entra applications_:
   create an application, give it the **Manager** role and add a key. Note
   its client ID and the key; the key is shown only once.
3. Note the tenant ID (_Account settings > Tenants_) and the seller ID
   (_Account settings > Legal info_ or _Identifiers_).
4. In GitHub, _Settings > Environments > store_ (only `v*` tags may use it),
   add the secrets `PARTNER_CENTER_TENANT_ID`, `PARTNER_CENTER_CLIENT_ID`,
   `PARTNER_CENTER_CLIENT_SECRET` and `PARTNER_CENTER_SELLER_ID`.
5. Set the repository variable `MSSTORE_AUTOSUBMIT` to `true`.

The key expires (at most after two years in Partner Center); add a new one
and replace `PARTNER_CENTER_CLIENT_SECRET` before it does.

The listing texts are in [STORE-LISTING.md](STORE-LISTING.md). The six
screenshots are composed from the real interface in
[`store/scenes.html`](store/scenes.html); with `npm run dev` running,
`node docs/store/render.mjs <path to chrome.exe>` renders them at 3840×2160
into `src-tauri/target/store-screenshots`.

The first submission also needs the listing, screenshots, the privacy policy
URL (`https://github.com/michu1810/limvo#privacy-and-security`), the age
rating questionnaire and a reason for the `runFullTrust` capability:
Limvo is a desktop app that reads the sign-ins of the AI tools the user has
on this computer and draws a reading over the Windows taskbar, which needs a
full-trust desktop process.
