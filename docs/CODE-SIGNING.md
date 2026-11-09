# Code signing

Limvo is set up for free code signing from the
[SignPath Foundation](https://signpath.org), which signs open source projects
that build their releases in public CI. Until the application is approved,
releases are unsigned and Windows SmartScreen shows a warning on download.

## Code signing policy

Once approved: free code signing provided by [SignPath.io](https://signpath.io),
certificate by [SignPath Foundation](https://signpath.org).

- Only installers built by the [Release workflow](../.github/workflows/release.yml)
  from a tag in this repository are signed. Nothing built on a personal
  machine is ever submitted.
- Committers and reviewers: [@michu1810](https://github.com/michu1810).
- Approvers: [@michu1810](https://github.com/michu1810). Each signing request is
  approved by hand in SignPath.
- Privacy: Limvo sends no data anywhere except the usage requests described
  in the [README](../README.md#privacy-and-security).

## Update signing (maintainers)

In-app updates are verified with a separate minisign key, unrelated to the
Authenticode certificate. The public key is in `src-tauri/tauri.conf.json`
(`plugins.updater.pubkey`). The private key and its password are secrets of
the `release` environment: `TAURI_SIGNING_PRIVATE_KEY` and
`TAURI_SIGNING_PRIVATE_KEY_PASSWORD`. Keep a backup of both outside this
machine: without them no installed copy accepts another update, and a new key
means every user reinstalls by hand.

The Release workflow signs the update after Authenticode signing, because the
signature has to cover the exact bytes people download, and writes
`latest.json` itself. Installed copies only see a release once its draft is
published.

## Turning it on (maintainers)

1. Apply at [signpath.org/apply](https://signpath.org/apply) with the public
   repository URL. SignPath reviews the project, its license and this policy.
2. After approval, in SignPath: create the project, a signing policy (for
   example `release-signing`) and connect the GitHub repository as a trusted
   build system. GitHub hands every artifact over as a ZIP, so the project's
   default artifact configuration needs a `<zip-file>` root. One
   configuration serves both requests of a release, the app and the
   installer:

   ```xml
   <?xml version="1.0" encoding="utf-8"?>
   <artifact-configuration xmlns="http://signpath.io/artifact-configuration/v1">
     <zip-file>
       <pe-file path="*.exe">
         <authenticode-sign />
       </pe-file>
     </zip-file>
   </artifact-configuration>
   ```
3. In GitHub, _Settings > Environments > release_ (not the repository-wide
   secrets: the `release` environment only serves runs from `v*` tags and
   waits for a maintainer's approval, so no branch or pull request can read
   these):
   - secret `SIGNPATH_API_TOKEN`: an API token of a SignPath CI user that may
     submit to the project;
   - variables `SIGNPATH_ORGANIZATION_ID`, `SIGNPATH_PROJECT_SLUG` and
     `SIGNPATH_SIGNING_POLICY_SLUG`.
4. Push a tag. The Release workflow builds the app, submits `limvo.exe`,
   waits for the approval and checks its signature. It then builds the
   installer around the signed app, submits the installer, waits for a second
   approval and checks that signature too. Only then does it sign the update,
   write the checksums of the signed file and draft the release with all of
   them. Each release therefore asks for two approvals in SignPath.

The app is signed before the installer is built because the installer is a
compressed archive: signing it covers only the outer file, never the app it
unpacks. The bundler would normally write the bundle type into the app while
packaging it, which breaks a signature, so the workflow writes it before
signing and packages with `--no-binary-patching`. The uninstaller that NSIS
generates stays unsigned.

Without the variables the signing steps are skipped, so forks and test tags
still produce an unsigned draft.
