//! Plugins. Within an account, a scope change rewrites the install entry of
//! `plugins/installed_plugins.json` and moves `enabledPlugins`; the cached files stay put.
//! To another account, Claude Code installs it itself: that registry format is internal.

use super::{obj, settings_file, Files, GONE, SHAPE};
use crate::model::{Node, Scope};
use crate::scan::ScanContext;
use serde_json::{json, Value};
use std::process::{Command, Stdio};

pub(super) fn transfer(n: &Node, from: &ScanContext, dest: &ScanContext, to: Scope, copy: bool) -> Result<(), String> {
    let id = n.meta["id"].as_str().ok_or(GONE)?;
    if dest.config_dir == from.config_dir {
        return scope(n, id, from, to, copy);
    }
    // Project and local installs belong to the project, not to an account.
    if n.scope != Scope::User || to != Scope::User {
        return Err("A plugin changes account only from and to the user scope.".into());
    }
    account(id, from, dest, copy)
}

fn scope(n: &Node, id: &str, ctx: &ScanContext, to: Scope, copy: bool) -> Result<(), String> {
    let registry = ctx.config_dir.join("plugins/installed_plugins.json");
    let (src_settings, dst_settings) = (settings_file(ctx, n.scope)?, settings_file(ctx, to)?);
    let project = |s: Scope| match s {
        Scope::User => None,
        _ => ctx.project.as_ref().map(|p| ctx.key(p)),
    };
    let mut files = Files::default();

    let installs = obj(files.get(&registry)?, &["plugins".into()])?
        .get_mut(id)
        .and_then(Value::as_array_mut)
        .ok_or(GONE)?;
    let at = |installs: &[Value], s: Scope| {
        let scope = json!(s);
        installs
            .iter()
            .position(|i| i.get("scope").unwrap_or(&json!("user")) == &scope && i["projectPath"].as_str() == project(s).as_deref())
    };
    if at(installs, to).is_some() {
        return Err(format!("{id} is already installed there."));
    }
    let src = at(installs, n.scope).ok_or(GONE)?;
    if copy {
        installs.push(installs[src].clone());
    }
    let entry = match copy {
        true => installs.last_mut(),
        false => installs.get_mut(src),
    };
    let entry = entry.and_then(Value::as_object_mut).ok_or(SHAPE)?;
    entry.insert("scope".into(), json!(to));
    match project(to) {
        Some(p) => entry.insert("projectPath".into(), json!(p)),
        None => entry.shift_remove("projectPath"),
    };

    // Only an existing source is touched: a copy never rewrites it, a move only drops the key.
    if !copy && src_settings.exists() {
        let enabled = files.get(&src_settings)?.get_mut("enabledPlugins").and_then(Value::as_object_mut);
        enabled.map(|m| m.shift_remove(id));
    }
    obj(files.get(&dst_settings)?, &["enabledPlugins".into()])?.insert(id.into(), json!(n.enabled));
    files.save(&format!("move plugin {id}"))
}

/// Installs the plugin on `dest`, adding its marketplace first when `dest` doesn't know it.
/// A move then uninstalls it from `from`, keeping its data directory.
fn account(id: &str, from: &ScanContext, dest: &ScanContext, copy: bool) -> Result<(), String> {
    let (_, market) = id.split_once('@').ok_or(GONE)?;
    if marketplaces(dest)[market].is_null() {
        let source = &marketplaces(from)[market]["source"];
        let spec = ["repo", "url", "path"].iter().find_map(|k| source[k].as_str());
        claude(dest, &["plugin", "marketplace", "add", spec.ok_or("Unknown marketplace source.")?])?;
    }
    claude(dest, &["plugin", "install", id, "--scope", "user"])?;
    if !copy {
        claude(from, &["plugin", "uninstall", id, "--scope", "user", "--keep-data"])?;
    }
    Ok(())
}

fn marketplaces(ctx: &ScanContext) -> Value {
    let text = std::fs::read_to_string(ctx.config_dir.join("plugins/known_marketplaces.json")).unwrap_or_default();
    serde_json::from_str(&text).unwrap_or_default()
}

/// `claude` on the account of `ctx`, with the login-shell PATH once the app knows it.
/// The default account is the one Claude Code picks with no `CLAUDE_CONFIG_DIR` at all.
// ponytail: an npm install on Windows is `claude.cmd`, which Command can't spawn; go through `cmd /C` if that bites.
fn claude(ctx: &ScanContext, args: &[&str]) -> Result<(), String> {
    let mut cmd = Command::new("claude");
    cmd.args(args).stdin(Stdio::null());
    match ctx.config_dir == ctx.home().join(".claude") {
        true => cmd.env_remove("CLAUDE_CONFIG_DIR"),
        false => cmd.env("CLAUDE_CONFIG_DIR", &ctx.config_dir),
    };
    if let Some(path) = crate::rules::search_path() {
        cmd.env("PATH", path);
    }
    #[cfg(windows)]
    std::os::windows::process::CommandExt::creation_flags(&mut cmd, 0x0800_0000); // CREATE_NO_WINDOW
    let out = cmd.output().map_err(|e| format!("claude: {e}"))?;
    if out.status.success() {
        return Ok(());
    }
    let msg = [out.stderr, out.stdout].concat();
    Err(format!("claude {}: {}", args.join(" "), String::from_utf8_lossy(&msg).trim()))
}
