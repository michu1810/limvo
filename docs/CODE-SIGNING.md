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

## Turning it on (maintainers)

1. Apply at [signpath.org/apply](https://signpath.org/apply) with the public
   repository URL. SignPath reviews the project, its license and this policy.
2. After approval, in SignPath: create the project, a signing policy (for
   example `release-signing`) and connect the GitHub repository as a trusted
   build system.
3. In GitHub, _Settings > Environments > release_ (not the repository-wide
   secrets: the `release` environment only serves runs from `v*` tags and
   waits for a maintainer's approval, so no branch or pull request can read
   these):
   - secret `SIGNPATH_API_TOKEN`: an API token of a SignPath CI user that may
     submit to the project;
   - variables `SIGNPATH_ORGANIZATION_ID`, `SIGNPATH_PROJECT_SLUG` and
     `SIGNPATH_SIGNING_POLICY_SLUG`.
4. Push a tag. The Release workflow builds the installer, submits it, waits for
   the approval, checks the Authenticode signature, replaces the draft asset
   with the signed one and publishes the checksums of the signed file.

Without the variables the signing steps are skipped, so forks and test tags
still produce an unsigned draft.
