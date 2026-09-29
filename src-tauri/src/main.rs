// Thin Tauri facade over the `superpebble` crate: typed IPC commands + a file watcher.
// The UI has no filesystem access of its own.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashSet;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Mutex;
use superpebble::{accounts, model::Graph, ScanContext};
use tauri::{AppHandle, Emitter, State};

#[derive(Default)]
struct AppState {
    watcher: Mutex<Option<RecommendedWatcher>>,
    /// Files the last scan found: the only ones `open_path` accepts.
    known: Mutex<HashSet<PathBuf>>,
}

#[tauri::command]
fn accounts() -> Vec<accounts::Account> {
    accounts::list()
}

// Account writes return the fresh list so the UI never shows a stale state.
#[tauri::command]
fn create_account(name: String, share: Vec<String>, alias: bool) -> Result<Vec<accounts::Account>, String> {
    accounts::create(&name, &share, alias).map(|_| accounts::list())
}

#[tauri::command]
fn set_shared(config_dir: PathBuf, item: String, on: bool) -> Result<Vec<accounts::Account>, String> {
    accounts::set_shared(&config_dir, &item, on).map(|_| accounts::list())
}

#[tauri::command]
fn set_alias(config_dir: PathBuf, on: bool) -> Result<Vec<accounts::Account>, String> {
    accounts::set_alias(&config_dir, on).map(|_| accounts::list())
}

#[tauri::command]
fn projects(config_dir: PathBuf) -> Vec<PathBuf> {
    accounts::projects(config_dir)
}

#[tauri::command]
async fn scan(state: State<'_, AppState>, config_dir: PathBuf, project: Option<PathBuf>) -> Result<Graph, String> {
    let g = superpebble::scan(&ScanContext { config_dir, project });
    *state.known.lock().unwrap() = g.nodes.iter().map(|n| n.source.clone()).collect();
    Ok(g)
}

/// Replaces the previous watcher; emits `config-changed` on any write under the watched paths.
#[tauri::command]
fn watch(app: AppHandle, state: State<AppState>, config_dir: PathBuf, project: Option<PathBuf>) -> Result<(), String> {
    let targets = ScanContext { config_dir, project }.watched_paths();
    let matches = targets.clone();
    let mut w = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        let Ok(e) = res else { return };
        // Claude Code reads these files constantly; only writes matter.
        if !matches!(e.kind, EventKind::Access(_)) && e.paths.iter().any(|p| matches.iter().any(|t| p.starts_with(t))) {
            let _ = app.emit("config-changed", ());
        }
    })
    .map_err(|e| e.to_string())?;
    for t in &targets {
        // Files are replaced by rename on save, so watch their parent dir rather than the inode.
        let res = if t.is_dir() {
            w.watch(t, RecursiveMode::Recursive)
        } else {
            match t.parent().filter(|p| p.is_dir()) {
                Some(p) => w.watch(p, RecursiveMode::NonRecursive),
                None => continue,
            }
        };
        res.map_err(|e| e.to_string())?;
    }
    *state.watcher.lock().unwrap() = Some(w);
    Ok(())
}

#[tauri::command]
fn open_path(state: State<AppState>, path: PathBuf) -> Result<(), String> {
    if !state.known.lock().unwrap().contains(&path) {
        return Err("path unknown to the last scan".into());
    }
    #[cfg(windows)]
    return open_windows(&path);
    #[cfg(not(windows))]
    Command::new("code")
        .arg(&path)
        .spawn()
        .or_else(|_| {
            if cfg!(target_os = "macos") {
                Command::new("open").arg("-t").arg(&path).spawn()
            } else {
                Command::new("xdg-open").arg(&path).spawn()
            }
        })
        .map(drop)
        .map_err(|e| e.to_string())
}

/// `code` is `code.cmd` on Windows, which Command can't spawn directly: go through cmd,
/// hidden, and fall back to Explorer's default app.
#[cfg(windows)]
fn open_windows(path: &std::path::Path) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let code = Command::new("cmd")
        .args(["/C", "code"])
        .arg(path)
        .creation_flags(CREATE_NO_WINDOW)
        .status();
    if code.is_ok_and(|s| s.success()) {
        return Ok(());
    }
    Command::new("explorer").arg(path).spawn().map(drop).map_err(|e| e.to_string())
}

/// Apps launched from Finder/a desktop menu get a minimal PATH, which would flag every
/// `npx`/`uvx` MCP as orphan. Asking the login shell takes seconds with a heavy zshrc,
/// so it runs off the startup path and triggers a rescan once known.
fn login_shell_path() -> Option<String> {
    if cfg!(windows) {
        return None; // GUI apps get the full user PATH on Windows
    }
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".into());
    let out = Command::new(shell)
        .args(["-ilc", "printf '\\n__SP__%s' \"$PATH\""])
        .stdin(std::process::Stdio::null())
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    let (_, path) = text.rsplit_once("__SP__")?;
    Some(path.trim().to_string()).filter(|p| !p.is_empty())
}

fn main() {
    tauri::Builder::default()
        .manage(AppState::default())
        .setup(|app| {
            let app = app.handle().clone();
            std::thread::spawn(move || {
                if let Some(path) = login_shell_path() {
                    superpebble::rules::set_search_path(path);
                    let _ = app.emit("config-changed", ());
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            accounts,
            create_account,
            set_shared,
            set_alias,
            projects,
            scan,
            watch,
            open_path
        ])
        .run(tauri::generate_context!())
        .expect("failed to start SuperPebble");
}
