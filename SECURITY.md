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
- Installer or update integrity.

## Design commitments

- Credentials are read only, kept in memory for one request, and never
  written, refreshed or logged.
- The interface has no direct access to the file system, the network or the
  credentials. It can only call the commands in `src-tauri/src/commands.rs`,
  under a strict content security policy.
- The only network destination is the provider's API over HTTPS.
