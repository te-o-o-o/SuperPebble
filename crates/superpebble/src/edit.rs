//! Edits made from the app: turning an element on or off, the cleanup, manual snapshots.
//! Every write goes through `transfer::Files`: a snapshot first, then atomic writes.

use crate::model::{Kind, Node, Scope};
use crate::scan::{scan, ScanContext};
use crate::transfer::{obj, settings_file, take_hook, take_mcp, Files, GONE, SHAPE};
use crate::{snapshot, wsl};
use serde_json::json;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn writable(ctx: &ScanContext) -> Result<(), String> {
    match wsl::root(&ctx.config_dir) {
        Some(_) => Err("WSL accounts are read-only.".into()),
        None => Ok(()),
    }
}

/// Plugins: `enabledPlugins` in the settings of their scope. MCP servers: the per-project
/// lists of `.claude.json`, the ones `/mcp` writes. Nothing else has an off switch.
pub fn set_enabled(ctx: &ScanContext, id: &str, on: bool) -> Result<(), String> {
    writable(ctx)?;
    let graph = scan(ctx);
    let n = graph.nodes.iter().find(|n| n.id == id).ok_or(GONE)?;
    if n.parent.is_some() || n.scope == Scope::Managed {
        return Err("Plugin content and managed settings cannot be turned off.".into());
    }
    let mut files = Files::default();
    match n.kind {
        Kind::Plugin => {
            let plugin = n.meta["id"].as_str().ok_or(GONE)?;
            obj(files.get(&settings_file(ctx, n.scope)?)?, &["enabledPlugins".into()])?.insert(plugin.into(), json!(on));
        }
        Kind::Mcp => {
            let project = ctx
                .project
                .as_ref()
                .ok_or("Pick a project: Claude Code turns MCP servers off per project.")?;
            let list = match n.scope {
                Scope::Project => "disabledMcpjsonServers",
                _ => "disabledMcpServers",
            };
            let names = obj(files.get(&ctx.claude_json())?, &["projects".into(), ctx.key(project)])?
                .entry(list)
                .or_insert_with(|| json!([]))
                .as_array_mut()
                .ok_or(SHAPE)?;
            names.retain(|v| v.as_str() != Some(&n.name));
            if !on {
                names.push(json!(n.name));
            }
        }
        _ => return Err("Claude Code has no off switch for this element.".into()),
    }
    files.save(&format!("turn {} {}", if on { "on" } else { "off" }, n.name))?;
    // A settings file of higher precedence may still decide otherwise.
    match scan(ctx).nodes.iter().find(|m| m.id == id) {
        Some(m) if m.enabled != on => Err("Another settings file overrides this: open it to change it there.".into()),
        _ => Ok(()),
    }
}

/// Removes the given orphan MCP servers and broken hooks, in one snapshot.
pub fn clean_up(ctx: &ScanContext, ids: &[String]) -> Result<(), String> {
    writable(ctx)?;
    let graph = scan(ctx);
    let fixable: Vec<&str> = graph
        .issues
        .iter()
        .filter(|i| i.fix)
        .flat_map(|i| &i.nodes)
        .map(String::as_str)
        .collect();
    let mut nodes: Vec<&Node> = graph
        .nodes
        .iter()
        .filter(|n| ids.contains(&n.id) && fixable.contains(&n.id.as_str()))
        .collect();
    // Hooks of one file by decreasing index: removing one shifts the indexes after it.
    nodes.sort_by_key(|n| std::cmp::Reverse((n.source.clone(), n.meta["index"][0].as_u64(), n.meta["index"][1].as_u64())));
    let mut files = Files::default();
    for n in &nodes {
        match n.kind {
            Kind::Mcp => drop(take_mcp(&mut files, n, ctx, false)?),
            Kind::Hook => drop(take_hook(&mut files, n, false)?),
            _ => {}
        }
    }
    files.save(&format!("clean up {} element(s)", nodes.len()))
}

/// Copies every config file of the account and project, plugin files aside.
pub fn snapshot_now(ctx: &ScanContext) -> Result<(), String> {
    let graph = scan(ctx);
    let files: BTreeSet<PathBuf> = graph
        .nodes
        .iter()
        .filter(|n| matches!(n.kind, Kind::Config | Kind::Mcp | Kind::Hook) && n.parent.is_none() && n.scope != Scope::Managed)
        .map(|n| n.source.clone())
        .chain([ctx.claude_json(), ctx.config_dir.join("plugins/installed_plugins.json")])
        .collect();
    let paths: Vec<&Path> = files.iter().map(PathBuf::as_path).collect();
    snapshot::save(&paths, "manual snapshot").map(drop).map_err(|e| e.to_string())
}
