//! Builds the graph of what Claude Code loads for one account (`CLAUDE_CONFIG_DIR`) and one project.
//! Read-only: nothing is executed, secret values never leave this module.

mod files;
mod mcp;
mod plugins;
mod settings;

use crate::model::*;
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
        if self.config_dir == home().join(".claude") && !inside.exists() {
            home().join(".claude.json")
        } else {
            inside
        }
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

pub fn home() -> PathBuf {
    std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default()
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

    pub fn issue(&mut self, rule: &'static str, severity: Severity, node: &str, message: String) {
        self.issues.push(Issue { rule, severity, nodes: vec![node.to_string()], message });
    }

    /// `Ok(None)` when the file doesn't exist; a parse error becomes an issue on a config node.
    pub fn read_json(&mut self, path: &Path, scope: Scope) -> Option<Value> {
        let text = std::fs::read_to_string(path).ok()?;
        match serde_json::from_str(&text) {
            Ok(v) => Some(v),
            Err(e) => {
                let name = file_name(path);
                let id = self.push(Kind::Config, scope, path, &name).id.clone();
                self.issue("invalid-json", Severity::Error, &id, format!("JSON illisible : {name} ({e})"));
                None
            }
        }
    }
}

pub fn scan(ctx: &ScanContext) -> Graph {
    let mut s = Scan { ctx, nodes: vec![], issues: vec![] };
    let st = settings::scan(&mut s);
    files::memory(&mut s);
    files::dir_items(&mut s, &ctx.config_dir, Scope::User, None);
    if let Some(p) = &ctx.project {
        files::dir_items(&mut s, &p.join(".claude"), Scope::Project, None);
    }
    mcp::scan(&mut s, &st);
    plugins::scan(&mut s, &st);
    if st.disable_all_hooks {
        s.nodes.iter_mut().filter(|n| n.kind == Kind::Hook).for_each(|n| n.enabled = false);
    }

    let mut issues = s.issues;
    issues.extend(crate::rules::check(&s.nodes));
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
