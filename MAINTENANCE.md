# Maintenance

Maintained fork: `0xble/railway-cli` of `railwayapp/cli`; branch
`feat/multiaccount`, upstream `master`. Canonical worktree:
`/Users/brianle/Repos/railway-cli-multiaccount`; accepted baseline
`7113bb8893612e4bcf092e145b0c920e674c2ce5`. Publish only to `fork`.

## Preserve

- Named-account selection happens before refresh, telemetry, updates, HTTP,
  WebSocket, SSH, MCP, or subprocess authentication. With one profile it is
  selected; with more, an invocation must name `--account`.
- Profiles live under `~/.railway/accounts`; legacy `config.json` is never
  changed by account import. Named profiles never use ambient Railway tokens.
- Installation is separate from source publication; do not replace a live
  Railway binary without separate authorization and SHA proof.

## Active patch: MULTIACCOUNT-001

- **Behavior:** named account profiles, account list/import, and isolated auth.
- **Surfaces:** `src/config.rs`, `src/main.rs`, `src/commands/account.rs`.
- **Upstream issue:** https://github.com/railwayapp/cli/issues/688
- **Regression:** `cargo test`, `cargo check`, and isolated-HOME CLI tests.
- **Rollback:** revert this patch commit.
- **Retire when:** upstream ships equivalent isolated named profiles.

## Update and verify

Fetch `origin/master` and `fork/master`, rebase or reconcile the patch, then
run formatter, focused regressions, full tests, and build. Before publication,
fetch upstream again and prove no unreviewed upstream-only commits; verify owned
remote SHA parity afterward.
