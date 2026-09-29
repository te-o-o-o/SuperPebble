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

/// One node per server. Only env/header *keys* are kept; values are inspected here and dropped.
pub fn servers(s: &mut Scan, map: Option<&Value>, scope: Scope, source: &Path, parent: Option<(&str, &Path)>) {
    let Some(map) = map.and_then(Value::as_object) else { return };
    for (name, cfg) in map {
        let env = cfg.get("env").and_then(Value::as_object);
        let headers = cfg.get("headers").and_then(Value::as_object);
        let keys = |m: Option<&serde_json::Map<String, Value>>| m.map(|m| m.keys().cloned().collect::<Vec<_>>()).unwrap_or_default();
        let secrets: Vec<&String> = env
            .into_iter()
            .chain(headers)
            .flatten()
            .filter(|(k, v)| is_plaintext_secret(k, v))
            .map(|(k, _)| k)
            .collect();

        let n = s.push(Kind::Mcp, scope, source, name);
        n.meta = json!({
            "type": cfg.get("type").and_then(Value::as_str).unwrap_or(if cfg.get("command").is_some() { "stdio" } else { "http" }),
            "command": cfg.get("command"),
            "args": cfg.get("args"),
            "url": cfg.get("url"),
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

fn is_plaintext_secret(key: &str, value: &Value) -> bool {
    let k = key.to_ascii_uppercase();
    let looks_secret = ["_TOKEN", "_KEY", "_SECRET", "PASSWORD"]
        .iter()
        .any(|s| k.ends_with(s) || k.contains(s))
        || k == "AUTHORIZATION"
        || k == "X-API-KEY";
    // `${VAR}` is a reference resolved at launch, not a secret on disk.
    looks_secret && value.as_str().is_some_and(|v| !v.is_empty() && !v.contains("${"))
}

#[cfg(test)]
mod tests {
    use super::is_plaintext_secret;
    use serde_json::json;

    #[test]
    fn secret_detection() {
        assert!(is_plaintext_secret("GITHUB_TOKEN", &json!("ghp_x")));
        assert!(is_plaintext_secret("Authorization", &json!("Bearer x")));
        assert!(!is_plaintext_secret("GITHUB_TOKEN", &json!("${GITHUB_TOKEN}")));
        assert!(!is_plaintext_secret("LOG_LEVEL", &json!("debug")));
    }
}
