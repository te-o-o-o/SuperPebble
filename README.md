<h1 align="center">SuperPebble</h1>

<p align="center">
  See everything plugged into Claude Code, on one hand-drawn map —<br>
  plugins, skills, MCP servers, hooks, agents, CLAUDE.md — and what's broken.
</p>

<p align="center"><sub>Rust + Tauri · macOS, Linux, Windows · MIT</sub></p>

<p align="center">
  <a href="#features">Features</a> ·
  <a href="#install">Install</a> ·
  <a href="#command-line">Command line</a> ·
  <a href="#several-accounts">Accounts</a> ·
  <a href="#development">Development</a> ·
  <a href="CHANGELOG.md">Changelog</a> ·
  <a href="README.fr.md">Français</a>
</p>

<p align="center">
  <img src="docs/screenshot.png" width="840" alt="SuperPebble: Claude Code in the middle, its skills, plugins, MCP servers, hooks, agents and config files branching out as hand-drawn pebbles, with the Skills branch open in the side panel">
</p>

Your Claude Code setup is spread across `~/.claude`, `~/.claude.json`,
`.claude/` in each project, `.mcp.json`, plugin caches and managed settings.
SuperPebble reads all of it and draws what Claude Code actually loads: one
pebble per item, one branch per kind, one badge per scope.

## Features

- **The whole config on one map**: plugins and what they bring, skills, MCP
  servers, hooks, agents, slash commands, `CLAUDE.md` and settings files.
- **Scopes at a glance**: user, project, local and managed, each with its badge,
  so you see where a skill or server really comes from.
- **Doctor**: flags orphan MCP servers (command not found), broken hooks
  (missing script), secrets in plain text, duplicates across scopes, invalid
  skills and unreadable JSON.
- **Token weight**: a rough estimate (chars/4) of what each file costs at startup,
  and `superpebble weight` for the total, by category, with the heaviest files.
- **Live**: a file watcher rescans as soon as a config file changes.
- **Several Claude Code accounts**: create `~/.claude-<name>` accounts, share
  skills, agents, commands or `CLAUDE.md` with the default one through symlinks,
  and get a `claude-<name>` alias in `~/.zshrc`.
- **Read-only by default**: the scan never runs an MCP server or a hook, and
  secret values never leave the scanner. Account changes only ever touch
  `~/.claude-<name>`, and `~/.zshrc` is snapshotted before any edit.

The interface is in English, or French when the system is; switch it from the top bar.

## Install

Download the installer for your system from the [latest release](https://github.com/te-o-o-o/SuperPebble/releases/latest):
`.exe` setup or portable `.zip` on Windows, universal `.dmg` on macOS,
`.AppImage` or `.deb` on Linux. Nothing is signed: Windows shows a SmartScreen
warning, and macOS needs `xattr -dr com.apple.quarantine SuperPebble.app` once.

Or build from source. You need Rust and Node:

```sh
npm install
npx tauri build --bundles app       # macOS: target/release/bundle/macos/SuperPebble.app
npx tauri build --bundles deb       # Linux, or appimage
```

Then open the app, or run it from a terminal to see its logs:

```sh
open target/release/bundle/macos/SuperPebble.app
./target/release/bundle/macos/SuperPebble.app/Contents/MacOS/superpebble-app
```

## Command line

The scanner is also a standalone CLI, no window needed. Install it with
`cargo install --path crates/superpebble`, or run it in place:

```sh
cargo run -p superpebble -- doctor      # problems for the current project
cargo run -p superpebble -- weight      # estimated startup context, by category
cargo run -p superpebble -- scan        # the graph as JSON
cargo run -p superpebble -- accounts    # accounts, sharing and shell aliases
```

`--config DIR` picks an account (default `$CLAUDE_CONFIG_DIR` or `~/.claude`),
`--project DIR` or `--no-project` the project (default: the current directory).
`doctor` exits 0 (clean), 1 (warnings) or 2 (errors) and takes `--json`,
`--severity warn|error` and `--quiet`, for CI. Every option and rule:
[crates/superpebble/README.md](crates/superpebble/README.md).

## Several accounts

An account is a config dir: `~/.claude` is the default one, `~/.claude-work`
is the account `work`. From **Manage accounts**, SuperPebble creates the dir,
links whatever you want shared with the default account, and adds the alias:

```sh
claude-work     # = CLAUDE_CONFIG_DIR=$HOME/.claude-work claude
```

MCP servers, plugins and credentials always stay per account.

## Development

```sh
# Linux / WSL: Tauri system dependencies, once
sudo apt install libwebkit2gtk-4.1-dev build-essential libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev

npm run tauri dev               # the app, with hot reload
cargo test -p superpebble
```

Without Tauri, `npm run fixture && npm run dev` serves the current directory's
graph in a browser.

```
crates/superpebble/   scan, rules and accounts, no Tauri (lib + `superpebble` CLI)
src-tauri/            Tauri shell: IPC commands, file watcher
src/                  React UI: React Flow + rough.js
```

Windows build, from Linux or WSL:

```sh
npx tauri build --target x86_64-pc-windows-gnu --no-bundle
```

Ship `superpebble-app.exe` together with `WebView2Loader.dll` from
`target/x86_64-pc-windows-gnu/release/`.

## Release

The version lives in one place, `[workspace.package]` in the root `Cargo.toml`:
the app and the crate inherit it. Bump it, date the `Unreleased` section of
[CHANGELOG.md](CHANGELOG.md), commit, then tag:

```sh
git tag v0.2.0 && git push --tags
cargo publish -p superpebble        # the CLI on crates.io, by hand
```

The `release` workflow builds the Windows installer and portable zip, the
universal macOS disk image and the Linux AppImage and `.deb`, then publishes
them on a GitHub Release. Binaries are not signed.

## License

MIT, see [LICENSE](LICENSE).
