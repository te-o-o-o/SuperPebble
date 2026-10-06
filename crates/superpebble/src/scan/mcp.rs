use super::secrets;
use super::settings::Settings;
use super::Scan;
use crate::model::{Kind, Scope};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::path::Path;

pub fn scan(s: &mut Scan, st: &Settings) {
    let cj_path = s.ctx.claude_json();
    let cj = s.read_json(&cj_path, Scope::User).unwrap_or_default();
    servers(s, cj.get("mcpServers"), Scope::User, &cj_path, None);

    let Some(project) = s.ctx.project.clone() else { return };
    let local = &cj["projects"][s.ctx.key(&project).as_str()];
    let disabled: HashSet<&str> = local["disabledMcpServers"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect();
    let before = s.nodes.len();
    servers(s, local.get("mcpServers"), Scope::Local, &cj_path, None);

    let mcp_json = project.join(".mcp.json");
    if let Some(v) = s.read_json(&mcp_json, Scope::Project) {
        servers(s, v.get("mcpServers"), Scope::Project, &mcp_json, None);
    }
    let rejected: HashSet<&str> = local["disabledMcpjsonServers"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect();
    for n in &mut s.nodes[before..] {
        let off = disabled.contains(n.name.as_str())
            || (n.scope == Scope::Project && (rejected.contains(n.name.as_str()) || st.disabled_mcpjson.contains(&n.name)));
        n.enabled &= !off;
    }
    // User-scope servers can also be toggled off per project via /mcp.
    for n in &mut s.nodes[..before] {
        if n.kind == Kind::Mcp && disabled.contains(n.name.as_str()) {
            n.enabled = false;
        }
    }
}

/// One node per server. Only env/header *keys* are kept; values are inspected here and dropped,
/// and secrets in args, command or url are masked.
pub fn servers(s: &mut Scan, map: Option<&Value>, scope: Scope, source: &Path, parent: Option<(&str, &Path)>) {
    let Some(map) = map.and_then(Value::as_object) else { return };
    for (name, cfg) in map {
        let env = cfg.get("env").and_then(Value::as_object);
        let headers = cfg.get("headers").and_then(Value::as_object);
        let keys = |m: Option<&serde_json::Map<String, Value>>| m.map(|m| m.keys().cloned().collect::<Vec<_>>()).unwrap_or_default();
        let mut secrets: Vec<String> = env
            .into_iter()
            .chain(headers)
            .flatten()
            .filter(|(k, v)| secrets::plaintext(k, v))
            .map(|(k, _)| k.clone())
            .collect();
        let mut args = cfg.get("args").cloned();
        if let Some(Value::Array(a)) = &mut args {
            secrets::mask_args(a, &mut secrets);
        }
        let command = cfg
            .get("command")
            .and_then(Value::as_str)
            .map(|c| secrets::mask_command(c, &mut secrets));
        let url = cfg.get("url").and_then(Value::as_str).map(|u| secrets::mask_url(u, &mut secrets));

        let n = s.push(Kind::Mcp, scope, source, name);
        n.meta = json!({
            "type": cfg.get("type").and_then(Value::as_str).unwrap_or(if cfg.get("command").is_some() { "stdio" } else { "http" }),
            "command": command,
            "args": args,
            "url": url,
            "env_keys": keys(env),
            "header_keys": keys(headers),
            "plaintext_secrets": secrets,
        });
        if let Some((id, root)) = parent {
            n.parent = Some(id.to_string());
            n.meta["plugin_root"] = json!(root);
        }
    }
}
