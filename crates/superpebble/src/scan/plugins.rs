use super::settings::{self, Settings};
use super::{files, mcp, Scan};
use crate::model::{Kind, Scope};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

/// Installed plugins (`<config>/plugins/installed_plugins.json`) and everything they ship.
pub fn scan(s: &mut Scan, st: &Settings) {
    let registry = s.ctx.config_dir.join("plugins/installed_plugins.json");
    let Some(v) = s.read_json(&registry, Scope::User) else { return };
    let project = s.ctx.project.as_ref().map(|p| p.to_string_lossy().into_owned());

    for (id, installs) in v["plugins"].as_object().into_iter().flatten() {
        for inst in installs.as_array().into_iter().flatten() {
            let scope = match inst["scope"].as_str() {
                Some("project") => Scope::Project,
                Some("local") => Scope::Local,
                Some("managed") => Scope::Managed,
                _ => Scope::User,
            };
            if scope != Scope::User && inst["projectPath"].as_str() != project.as_deref() {
                continue;
            }
            let Some(root) = inst["installPath"].as_str().map(PathBuf::from) else {
                continue;
            };
            plugin(s, id, inst, &root, scope, st.enabled_plugins.get(id) == Some(&true));
        }
    }
}

fn plugin(s: &mut Scan, id: &str, inst: &Value, root: &Path, scope: Scope, enabled: bool) {
    let manifest_path = root.join(".claude-plugin/plugin.json");
    let manifest = s.read_json(&manifest_path, scope).unwrap_or_default();
    let (name, marketplace) = id.split_once('@').unwrap_or((id, ""));
    let first_child = s.nodes.len() + 1;
    let n = s.push(Kind::Plugin, scope, &manifest_path, name);
    n.enabled = enabled;
    n.meta = json!({
        "id": id,
        "marketplace": marketplace,
        "version": inst["version"],
        "description": manifest["description"],
        "root": root,
    });
    let pid = n.id.clone();

    files::dir_items(s, root, scope, Some(&pid));
    // `hooks` / `mcpServers` in the manifest: a path relative to the root, or inline.
    let (hooks, hooks_src) = inline_or_file(s, &manifest["hooks"], root, "hooks/hooks.json", scope);
    settings::hooks(s, hooks.get("hooks").or(Some(&hooks)), scope, &hooks_src, Some((&pid, root)));
    let (mcps, mcp_src) = inline_or_file(s, &manifest["mcpServers"], root, ".mcp.json", scope);
    mcp::servers(s, mcps.get("mcpServers").or(Some(&mcps)), scope, &mcp_src, Some((&pid, root)));

    let children = &mut s.nodes[first_child..];
    let mut total = 0;
    for c in children.iter_mut() {
        c.enabled &= enabled;
        total += c.tokens.unwrap_or(0);
    }
    s.nodes[first_child - 1].tokens = Some(total);
}

fn inline_or_file(s: &mut Scan, decl: &Value, root: &Path, default: &str, scope: Scope) -> (Value, PathBuf) {
    let manifest = root.join(".claude-plugin/plugin.json");
    match decl {
        Value::Object(_) => (decl.clone(), manifest),
        Value::String(rel) => {
            let p = root.join(rel);
            (s.read_json(&p, scope).unwrap_or_default(), p)
        }
        _ => {
            let p = root.join(default);
            (s.read_json(&p, scope).unwrap_or_default(), p)
        }
    }
}
