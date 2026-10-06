//! Builds the graph of what Claude Code loads for one account (`CLAUDE_CONFIG_DIR`) and one project.
//! Read-only: nothing is executed, secret values never leave this module.

mod files;
mod mcp;
mod plugins;
mod secrets;
mod settings;

use crate::model::*;
use crate::wsl;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct ScanContext {
    pub config_dir: PathBuf,
    pub project: Option<PathBuf>,
}

impl ScanContext {
    /// Same resolution as Claude Code: `$CLAUDE_CONFIG_DIR`, else `~/.claude`.
    pub fn default_config_dir() -> PathBuf {
        std::env::var_os("CLAUDE_CONFIG_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| home().join(".claude"))
    }

    /// `~/.claude.json` for the default account, `<config>/.claude.json` otherwise.
    pub fn claude_json(&self) -> PathBuf {
        let inside = self.config_dir.join(".claude.json");
        if self.config_dir == self.home().join(".claude") && !inside.exists() {
            self.home().join(".claude.json")
        } else {
            inside
        }
    }

    /// The account's `~`: the user's home inside the distro for a WSL account.
    pub fn home(&self) -> PathBuf {
        match wsl::root(&self.config_dir) {
            Some(_) => self.config_dir.parent().map(Path::to_path_buf).unwrap_or_default(),
            None => home(),
        }
    }

    /// An absolute path read from a config file, as this OS opens it.
    pub fn path(&self, p: &str) -> PathBuf {
        match wsl::root(&self.config_dir) {
            Some(root) if p.starts_with('/') => wsl::to_windows(&root, p),
            _ => PathBuf::from(p),
        }
    }

    /// `p` as config files spell it, e.g. the key of a project in `.claude.json`.
    pub fn key(&self, p: &Path) -> String {
        match wsl::root(&self.config_dir) {
            Some(root) => wsl::to_linux(&root, p),
            None => p.to_string_lossy().into_owned(),
        }
    }

    /// `<project>/.claude`, unless it is the account dir itself (Claude Code run from `~`):
    /// scanning it a second time as the project would flag every item as a duplicate.
    pub(crate) fn project_claude(&self) -> Option<PathBuf> {
        let d = self.project.as_ref()?.join(".claude");
        let same = d
            .canonicalize()
            .ok()
            .zip(self.config_dir.canonicalize().ok())
            .is_some_and(|(a, b)| a == b);
        (!same).then_some(d)
    }

    /// Files and dirs whose change should trigger a rescan. Dirs are watched recursively.
    pub fn watched_paths(&self) -> Vec<PathBuf> {
        let c = &self.config_dir;
        let mut v = vec![
            c.join("settings.json"),
            c.join("CLAUDE.md"),
            c.join("skills"),
            c.join("agents"),
            c.join("commands"),
            c.join("plugins/installed_plugins.json"),
            self.claude_json(),
        ];
        if let Some(p) = &self.project {
            v.extend([
                p.join(".claude"),
                p.join(".mcp.json"),
                p.join("CLAUDE.md"),
                p.join("CLAUDE.local.md"),
            ]);
        }
        v
    }
}

/// `$HOME`, or `%USERPROFILE%` on Windows (where Claude Code keeps `.claude` too).
pub fn home() -> PathBuf {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_default()
}

/// Collects nodes and scan-time issues (unreadable files, invalid skills).
pub(crate) struct Scan<'a> {
    pub ctx: &'a ScanContext,
    pub nodes: Vec<Node>,
    pub issues: Vec<Issue>,
}

impl Scan<'_> {
    pub fn push(&mut self, kind: Kind, scope: Scope, source: &Path, name: &str) -> &mut Node {
        self.nodes.push(Node {
            id: format!("{kind:?}:{scope:?}:{}#{name}", source.display()).to_lowercase(),
            kind,
            name: name.to_string(),
            scope,
            source: source.to_path_buf(),
            enabled: true,
            parent: None,
            tokens: None,
            modified: mtime(source),
            meta: json!({}),
        });
        self.nodes.last_mut().unwrap()
    }

    pub fn issue(&mut self, rule: &'static str, severity: Severity, node: &str, args: &[&str]) {
        let args = args.iter().map(|s| s.to_string()).collect();
        self.issues.push(Issue::new(rule, severity, vec![node.to_string()], args));
    }

    /// `None` when the file doesn't exist; a read or parse error becomes an issue on a config node.
    pub fn read_json(&mut self, path: &Path, scope: Scope) -> Option<Value> {
        let parsed = match std::fs::read_to_string(path) {
            Ok(text) => serde_json::from_str(&text).map_err(|e| e.to_string()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return None,
            Err(e) => Err(e.to_string()),
        };
        match parsed {
            Ok(v) => Some(v),
            Err(e) => {
                let name = file_name(path);
                let id = self.push(Kind::Config, scope, path, &name).id.clone();
                self.issue("invalid-json", Severity::Error, &id, &[&name, &e]);
                None
            }
        }
    }
}

pub fn scan(ctx: &ScanContext) -> Graph {
    let mut s = Scan {
        ctx,
        nodes: vec![],
        issues: vec![],
    };
    let st = settings::scan(&mut s);
    files::memory(&mut s);
    files::dir_items(&mut s, &ctx.config_dir, Scope::User, None);
    if let Some(d) = ctx.project_claude() {
        files::dir_items(&mut s, &d, Scope::Project, None);
    }
    mcp::scan(&mut s, &st);
    plugins::scan(&mut s, &st);
    if st.disable_all_hooks {
        s.nodes.iter_mut().filter(|n| n.kind == Kind::Hook).for_each(|n| n.enabled = false);
    }

    let mut issues = s.issues;
    issues.extend(crate::rules::check(&s.nodes, ctx));
    Graph {
        config_dir: ctx.config_dir.clone(),
        project: ctx.project.clone(),
        scanned_at: SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs()),
        nodes: s.nodes,
        issues,
    }
}

pub(crate) fn mtime(p: &Path) -> Option<u64> {
    let t = std::fs::metadata(p).ok()?.modified().ok()?;
    Some(t.duration_since(UNIX_EPOCH).ok()?.as_secs())
}

pub(crate) fn file_name(p: &Path) -> String {
    p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()
}

/// chars/4 estimate: a real tokenizer would need the network.
pub(crate) fn tokens(text: &str) -> u32 {
    (text.chars().count() as u32).div_ceil(4)
}
