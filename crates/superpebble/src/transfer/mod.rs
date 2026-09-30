//! Moves or copies one element to another account and/or scope.
//! Skills, agents and commands are files: renamed, or copied then deleted across disks.
//! MCP servers and hooks are JSON entries: removed and re-added, after a snapshot of every file touched.
//! Plugins: see `plugin`.

mod plugin;

use crate::model::{Kind, Node, Scope};
use crate::scan::{scan, ScanContext};
use crate::{snapshot, wsl};
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

pub(super) const NO_SCOPE: &str = "This element has no such scope.";
pub(super) const GONE: &str = "Element not found: rescan and retry.";
pub(super) const SHAPE: &str = "Unexpected JSON shape, left untouched.";

/// `id` is a node of the scan of `from`; project scopes stay on `from`'s project.
pub fn transfer(from: &ScanContext, id: &str, to_config_dir: &Path, to: Scope, copy: bool) -> Result<(), String> {
    let dest = ScanContext {
        config_dir: to_config_dir.to_path_buf(),
        project: from.project.clone(),
    };
    if wsl::root(&from.config_dir).is_some() || wsl::root(&dest.config_dir).is_some() {
        return Err("WSL accounts are read-only.".into());
    }
    let graph = scan(from);
    let n = graph.nodes.iter().find(|n| n.id == id).ok_or(GONE)?;
    if n.parent.is_some() || n.scope == Scope::Managed {
        return Err("Plugin content and managed settings cannot be moved.".into());
    }
    match n.kind {
        Kind::Skill | Kind::Agent | Kind::Command => file(n, from, &dest, to, copy),
        Kind::Mcp => mcp(n, from, &dest, to, copy),
        Kind::Hook => hook(n, &dest, to, copy),
        Kind::Plugin => plugin::transfer(n, from, &dest, to, copy),
        _ => Err("This element cannot be moved yet.".into()),
    }
}

/// `<config>` for user items, `<project>/.claude` for project ones.
fn items_dir(ctx: &ScanContext, scope: Scope) -> Result<PathBuf, String> {
    match (scope, &ctx.project) {
        (Scope::User, _) => Ok(ctx.config_dir.clone()),
        (Scope::Project, Some(p)) => Ok(p.join(".claude")),
        _ => Err(NO_SCOPE.into()),
    }
}

fn file(n: &Node, from: &ScanContext, dest: &ScanContext, to: Scope, copy: bool) -> Result<(), String> {
    let sub = match n.kind {
        Kind::Skill => "skills",
        Kind::Agent => "agents",
        _ => "commands",
    };
    // A skill is its whole folder.
    let src = match n.source.ends_with("SKILL.md") {
        true => n.source.parent().unwrap_or(&n.source).to_path_buf(),
        false => n.source.clone(),
    };
    let rel = src.strip_prefix(items_dir(from, n.scope)?.join(sub)).map_err(|_| GONE)?;
    let dst = items_dir(dest, to)?.join(sub).join(rel);
    if dst.exists() {
        return Err(format!("{} already exists.", dst.display()));
    }
    let io = |e: std::io::Error| e.to_string();
    fs::create_dir_all(dst.parent().unwrap_or(&dst)).map_err(io)?;
    if copy {
        return copy_all(&src, &dst).map_err(io);
    }
    // Rename fails across disks (Windows ↔ WSL): copy, then delete only once the copy is whole.
    fs::rename(&src, &dst)
        .or_else(|_| copy_all(&src, &dst).and_then(|_| remove_all(&src)))
        .map_err(io)
}

fn copy_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !src.is_dir() {
        return fs::copy(src, dst).map(drop);
    }
    fs::create_dir_all(dst)?;
    for e in fs::read_dir(src)? {
        let e = e?;
        copy_all(&e.path(), &dst.join(e.file_name()))?;
    }
    Ok(())
}

fn remove_all(p: &Path) -> std::io::Result<()> {
    match p.is_dir() {
        true => fs::remove_dir_all(p),
        false => fs::remove_file(p),
    }
}

/// The file and the object path holding the servers of a scope.
fn mcp_slot(ctx: &ScanContext, scope: Scope) -> Result<(PathBuf, Vec<String>), String> {
    let servers = "mcpServers".to_string();
    match (scope, &ctx.project) {
        (Scope::User, _) => Ok((ctx.claude_json(), vec![servers])),
        (Scope::Local, Some(p)) => Ok((ctx.claude_json(), vec!["projects".into(), ctx.key(p), servers])),
        (Scope::Project, Some(p)) => Ok((p.join(".mcp.json"), vec![servers])),
        _ => Err(NO_SCOPE.into()),
    }
}

fn mcp(n: &Node, from: &ScanContext, dest: &ScanContext, to: Scope, copy: bool) -> Result<(), String> {
    let (src, src_at) = mcp_slot(from, n.scope)?;
    let (dst, dst_at) = mcp_slot(dest, to)?;
    if (&src, &src_at) == (&dst, &dst_at) {
        return Err("Already there.".into());
    }
    let mut files = Files::default();
    let servers = obj(files.get(&src)?, &src_at)?;
    let cfg = match copy {
        true => servers.get(&n.name).cloned(),
        false => servers.shift_remove(&n.name),
    }
    .ok_or(GONE)?;
    let target = obj(files.get(&dst)?, &dst_at)?;
    if target.contains_key(&n.name) {
        return Err(format!("{} already exists there.", n.name));
    }
    target.insert(n.name.clone(), cfg);
    files.save(&format!("move mcp {}", n.name))
}

pub(super) fn settings_file(ctx: &ScanContext, scope: Scope) -> Result<PathBuf, String> {
    match (scope, &ctx.project) {
        (Scope::User, _) => Ok(ctx.config_dir.join("settings.json")),
        (Scope::Project, Some(p)) => Ok(p.join(".claude/settings.json")),
        (Scope::Local, Some(p)) => Ok(p.join(".claude/settings.local.json")),
        _ => Err(NO_SCOPE.into()),
    }
}

fn hook(n: &Node, dest: &ScanContext, to: Scope, copy: bool) -> Result<(), String> {
    let dst = settings_file(dest, to)?;
    if dst == n.source {
        return Err("Already there.".into());
    }
    let event = n.meta["event"].as_str().ok_or(GONE)?;
    let [i, j] = [0, 1].map(|k| n.meta["index"][k].as_u64().unwrap_or(u64::MAX) as usize);
    let mut files = Files::default();

    let hooks = obj(files.get(&n.source)?, &["hooks".into()])?;
    let groups = hooks.get_mut(event).and_then(Value::as_array_mut).ok_or(GONE)?;
    let group = groups.get_mut(i).ok_or(GONE)?;
    let matcher = group.get("matcher").cloned();
    let entries = group["hooks"].as_array_mut().filter(|e| j < e.len()).ok_or(GONE)?;
    let h = match copy {
        true => entries[j].clone(),
        false => entries.remove(j),
    };
    // Leave no empty group or event behind.
    if entries.is_empty() {
        groups.remove(i);
    }
    if groups.is_empty() {
        hooks.shift_remove(event);
    }

    let groups = obj(files.get(&dst)?, &["hooks".into()])?
        .entry(event)
        .or_insert_with(|| json!([]))
        .as_array_mut()
        .ok_or(SHAPE)?;
    let at = match groups.iter().position(|g| g.get("matcher") == matcher.as_ref()) {
        Some(at) => at,
        None => {
            groups.push(match &matcher {
                Some(m) => json!({ "matcher": m, "hooks": [] }),
                None => json!({ "hooks": [] }),
            });
            groups.len() - 1
        }
    };
    let entries = groups[at]["hooks"].as_array_mut().ok_or(SHAPE)?;
    if entries.contains(&h) {
        return Err("This hook already exists there.".into());
    }
    entries.push(h);
    files.save(&format!("move hook {event}"))
}

/// JSON files edited together: one read, one snapshot and one write each, even when the
/// source and the target are the same file.
#[derive(Default)]
pub(super) struct Files(BTreeMap<PathBuf, Value>);

impl Files {
    /// A missing file starts empty; an unreadable one stops everything.
    pub(super) fn get(&mut self, p: &Path) -> Result<&mut Value, String> {
        if !self.0.contains_key(p) {
            let v = match fs::read_to_string(p) {
                Ok(text) => serde_json::from_str(&text).map_err(|e| format!("{}: {e}", p.display()))?,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => json!({}),
                Err(e) => return Err(format!("{}: {e}", p.display())),
            };
            self.0.insert(p.to_path_buf(), v);
        }
        Ok(self.0.get_mut(p).expect("inserted above"))
    }

    pub(super) fn save(self, reason: &str) -> Result<(), String> {
        let paths: Vec<&Path> = self.0.keys().map(PathBuf::as_path).collect();
        snapshot::save(&paths, reason).map_err(|e| e.to_string())?;
        for (p, v) in &self.0 {
            let text = serde_json::to_string_pretty(v).map_err(|e| e.to_string())? + "\n";
            fs::create_dir_all(p.parent().unwrap_or(p)).map_err(|e| e.to_string())?;
            snapshot::write_atomic(p, &text).map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}

/// The object at `path`, created when missing. Never replaces a value of another type.
pub(super) fn obj<'a>(v: &'a mut Value, path: &[String]) -> Result<&'a mut Map<String, Value>, String> {
    let mut v = v;
    for k in path {
        v = v.as_object_mut().ok_or(SHAPE)?.entry(k.as_str()).or_insert_with(|| json!({}));
    }
    v.as_object_mut().ok_or_else(|| SHAPE.into())
}
