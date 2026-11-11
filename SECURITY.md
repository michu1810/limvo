# Security policy

Limvo handles sign-in tokens of other applications, so security reports
get priority.

## Reporting a vulnerability

Please report privately through GitHub: open the **Security** tab of this
repository and choose **Report a vulnerability**. Do not open a public issue
for anything that could expose a credential.

Include the Limvo version, your Windows version, and the steps to
reproduce. You can expect a first answer within a week.

## What is in scope

- Any way a token could leave the machine other than in the usage request to
  the provider it belongs to.
- Tokens appearing in logs, settings, crash output or the interface.
- Ways for web content or another process to call Limvo's commands.
- Installer or update integrity, including the update feed and its signatures.

## Design commitments

- Credentials are read only, kept in memory for one request, and never
  written, refreshed or logged.
- The interface has no direct access to the file system, the network or the
  credentials. It can only call the commands in `src-tauri/src/commands.rs`,
  under a strict content security policy.
- The folder of an extra account is picked in the system folder dialog,
  opened by the core. The interface names only the service and a label; no
  command accepts a path from it. The core checks that the folder holds the
  service's sign-in file without returning its contents.
- The only network destinations are the usage APIs of the signed-in services
  (`api.anthropic.com`, `chatgpt.com`, `api2.cursor.sh`,
  `cloudcode-pa.googleapis.com`), over HTTPS, each with its own token, and the
  update feed on GitHub Releases (`github.com`, served from
  `objects.githubusercontent.com`), which gets no token at all.
- Updates install only when their minisign signature matches the public key
  in `src-tauri/tauri.conf.json`. The private key lives only in the `release`
  environment of this repository, which serves `v*` tag runs after the
  maintainer approves them.
- Other apps' stores are opened read-only: Cursor's settings database with
  SQLite's read-only flags, the Credential Manager entries of Antigravity and
  the GitHub CLI with `CredReadW`, and the CLI's `hosts.yml` as plain text.
  Nothing is written back.
