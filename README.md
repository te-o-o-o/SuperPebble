# SuperPebble

A desktop app that draws everything plugged into Claude Code (plugins, skills, MCP servers, hooks, agents, config files) as a hand-drawn graph, and flags what is broken or duplicated. It also manages several Claude Code accounts.

Read-only by default: the scan never executes an MCP server or a hook, and secret values never leave the scanner.

## Layout

```
crates/superpebble/   scan, rules and accounts, no Tauri (lib + `superpebble` CLI)
src-tauri/            Tauri shell: IPC commands, file watcher
src/                  React UI: React Flow + rough.js
```

## Development

```sh
# Linux / WSL: Tauri system dependencies, once
sudo apt install libwebkit2gtk-4.1-dev build-essential libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev

npm install
npm run tauri dev                         # the app
cargo run -p superpebble -- doctor        # problems for the current project
cargo run -p superpebble -- scan          # the graph as JSON
cargo run -p superpebble -- accounts      # accounts, sharing and shell aliases
cargo test -p superpebble
```

Without Tauri, `npm run fixture && npm run dev` serves the current directory's graph in a browser.

## Windows build (from Linux / WSL)

```sh
npx tauri build --target x86_64-pc-windows-gnu --no-bundle
```

Ship `superpebble-app.exe` together with `WebView2Loader.dll` from `target/x86_64-pc-windows-gnu/release/`.
