# Named profiles

Read on every maintenance run after the [root contract](../MAINTENANCE.md).
Apply its shared invariants and update rules before evaluating these records.
Preserve the behavior below during reconciliation and require its focused proof
before changing or retiring the patch. The root's overall verification also applies.

## MULTIACCOUNT-001

- Named accounts, local list/import, account-scoped saved connections, Claude
  cache and agent preferences, MCP child argv pinning, private no-clobber import.
- Upstream issue: https://github.com/railwayapp/cli/issues/688
- Tests: `tests/multiaccount.rs`, `tests/multiaccount_auth.rs` (loopback HTTP),
  `tests/account_errors.rs` (portable JSON/path/ordering contract).
- The upstream 5.52.0 code-connection archive was reconciled using
  `Configs::account_data_dir_in(home)` for every archive/snapshot path.
- Do not automatically migrate ancillary state, SSH keys, or editor credentials.
- Retire when upstream supplies equivalent isolated named profiles.
