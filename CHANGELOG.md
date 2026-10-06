# Changelog

## 0.2.0 (2026-10-06)

### Added
- Context budget: click the Claude Code pebble for the estimated startup context,
  by category, with the 10 heaviest files; `superpebble weight` prints the same.
- `superpebble doctor --json`, `--severity warn|error` and `--quiet`.
- Secrets are detected in MCP args, command and url, and in hook commands, plus
  well-known token formats (`sk-…`, `ghp_…`, `AKIA…`) whatever their key.
- The UI follows the system language until one is picked.

### Changed
- `doctor` exit codes: 0 no problem, 1 warnings only, 2 errors, 64 bad usage
  (was 1 on errors, 2 on bad usage).
- One version number, `[workspace.package]` in the root `Cargo.toml`.

### Fixed
- Secret values in MCP args or urls were shown as is in the graph and the UI.
- Run from `~`, every user skill, agent and command was flagged as a duplicate.
- `broken-hook` no longer flags `VAR=value` prefixes, shell builtins, or files
  a redirection creates.
- `plaintext-secret` no longer flags `TOKEN_LIMIT`, `*_KEY_FILE`, `*_PUBLIC_KEY`
  or paths to key files.
- A config file that exists but cannot be read is reported instead of skipped.

## 0.1.0 (2026-10-04)

First release: the config graph, Doctor, token weight, accounts, moving items
across scopes and accounts, installers for Windows, macOS and Linux.
