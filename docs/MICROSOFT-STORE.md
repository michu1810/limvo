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
   artifact of the run. Download it and upload the `.msix` in a new Store
   submission.

The first submission also needs the listing, screenshots, the privacy policy
URL (`https://github.com/michu1810/limvo#privacy-and-security`), the age
rating questionnaire and a reason for the `runFullTrust` capability:
Limvo is a desktop app that reads the sign-ins of the AI tools the user has
on this computer and draws a reading over the Windows taskbar, which needs a
full-trust desktop process.
