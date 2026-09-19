# Maintenance

## Background

Maintained fork: `0xble/railway-cli` of `railwayapp/cli`; owned branch `master`,
upstream `origin/master`. Canonical source: `/Users/brianle/Repos/railway-cli`.
Working implementation: `/Users/brianle/Repos/railway-cli-multiaccount`.
Accepted upstream baseline: `17c8e2d` (5.52.0); publish only to `fork`.

## Preserve

- Named-account selection precedes refresh, telemetry, updates, HTTP, WebSocket,
  SSH, MCP, and subprocess authentication. Zero profiles retains legacy;
  one auto-selects; multiple require `--account`. Unknown explicit names fail.
- `~/.railway/accounts` profiles never use ambient Railway tokens. Imports do
  not change legacy config. Listing is alphabetical metadata, never secrets.
- Source publication and installation are separate. Installation requires
  authorization, rollback artifact, SHA proof, and unchanged live auth bytes.

## Active patches

Read both linked support files on every maintenance run. This root is the sole
enrolled contract; support files extend its shared Preserve and Update and verify
requirements with the complete patch records and focused proof.

| Patch | Required behavior and record | When |
| --- | --- | --- |
| MULTIACCOUNT-001 | [Isolated named profiles and account-scoped state](maintenance/named-profiles.md) | Every run |
| ACCOUNT-HARDENING-002 | [Credential-store, refresh, MCP, and fork-update protections](maintenance/account-hardening.md) | Every run |

## Update and verify

Fetch `origin/master` and `fork/master`; reconcile upstream before publication.
Run `cargo fmt --all --check`, `git diff --check`, `cargo test --locked`,
`cargo check --locked`, `cargo clippy --locked`, and release build. Review the
exact candidate once with independent-context evidence; disclose provider
fallbacks. Verify PR state and remote SHA after landing. Build the landed source
for installation so its embedded source commit identifies the installed tree.

Install the fork release with a preserved package-entry rollback and immutable
SHA-addressed executable, without editing auth. Never use upstream npm/brew
upgrade as the fork maintenance path. Runtime update protection cannot prevent
a human explicitly reinstalling an upstream package.
