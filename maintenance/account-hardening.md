# Account hardening

Read on every maintenance run after the [root contract](../MAINTENANCE.md).
Apply its shared invariants and update rules before evaluating these records.
Preserve the behavior below during reconciliation and require its focused proof
before changing or retiring the patch. The root's overall verification also applies.

## ACCOUNT-HARDENING-002

- Fork identity and upstream replacement policy are compile-time Cargo metadata.
  Both detached self-update and explicit package-manager upgrade are blocked.
  `railway upgrade --check` exposes fork identity and source commit. Removing
  the fork metadata restores upstream policy; no runtime bypass exists.
- Config reads fail closed on corruption and I/O failures without overwriting
  original bytes. Diagnostics omit deserializer values. Restore a known-good
  private backup rather than deleting the damaged store automatically.
- Session lock failures block refresh. Separate short write locks serialize
  credential merge and atomic publication without recursively locking refresh.
- Local MCP reloads its pinned config and discards the bearer-bearing client
  after logout, file removal, or unreadable config. In-flight requests already
  sent cannot be recalled.
- JSON account-selection errors use ACCOUNT_REQUIRED, ACCOUNT_UNKNOWN and
  ACCOUNT_INVALID. The parsed --json flag controls output, not child argv.
- Existing Linux/macOS/Windows hosted matrix explicitly runs portable account
  and store regressions. A missing/billing-blocked run is never called passing.
- Windows MSVC/GNU binaries reserve an 8 MiB main stack for the async dispatcher;
  the debug executable otherwise overflows before even parsing `--help`.
  Portable subprocess tests exercise the actual debug binary without skipping.
- Debug-only `RAILWAY_TEST_HOME` isolates subprocess credential fixtures on
  Windows, whose known-folder API ignores HOME/USERPROFILE. It must be absolute
  and is compiled out of release builds; release credential routing is unchanged.
  The mutating Windows subprocess fixtures run only in debug mode; release-mode
  tests must never fall back to a developer's real known-folder credentials.
- Tests: config lock/contention/redaction unit tests; config_store subprocess
  tests; actual loopback MCP no-Authorization regression; fork updater unit and
  package-manager subprocess refusal tests.
- Retire each behavior when upstream has equivalent tested protections.
