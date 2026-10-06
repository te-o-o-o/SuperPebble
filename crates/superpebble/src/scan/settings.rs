use super::{file_name, secrets, Scan};
use crate::model::{Kind, Scope};
use crate::wsl;
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Settings merged across scopes, lowest precedence first.
#[derive(Default)]
pub struct Settings {
    pub enabled_plugins: HashMap<String, bool>,
    pub disabled_mcpjson: HashSet<String>,
    pub disable_all_hooks: bool,
}

fn managed_path() -> PathBuf {
    if cfg!(target_os = "macos") {
        PathBuf::from("/Library/Application Support/ClaudeCode/managed-settings.json")
    } else if cfg!(windows) {
        PathBuf::from(r"C:\ProgramData\ClaudeCode\managed-settings.json")
    } else {
        PathBuf::from("/etc/claude-code/managed-settings.json")
    }
}

pub fn scan(s: &mut Scan) -> Settings {
    let mut files = vec![(Scope::User, s.ctx.config_dir.join("settings.json"))];
    if let Some(p) = &s.ctx.project {
        if let Some(d) = s.ctx.project_claude() {
            files.push((Scope::Project, d.join("settings.json")));
        }
        files.push((Scope::Local, p.join(".claude/settings.local.json")));
    }
    let managed = match wsl::root(&s.ctx.config_dir) {
        Some(_) => s.ctx.path("/etc/claude-code/managed-settings.json"),
        None => managed_path(),
    };
    files.push((Scope::Managed, managed));

    let mut out = Settings::default();
    for (scope, path) in files {
        let Some(v) = s.read_json(&path, scope) else { continue };
        s.push(Kind::Config, scope, &path, &file_name(&path));
        hooks(s, v.get("hooks"), scope, &path, None);
        if let Some(m) = v.get("enabledPlugins").and_then(Value::as_object) {
            for (id, on) in m {
                out.enabled_plugins.insert(id.clone(), on.as_bool().unwrap_or(false));
            }
        }
        if let Some(a) = v.get("disabledMcpjsonServers").and_then(Value::as_array) {
            out.disabled_mcpjson.extend(a.iter().filter_map(Value::as_str).map(String::from));
        }
        out.disable_all_hooks |= v.get("disableAllHooks").and_then(Value::as_bool).unwrap_or(false);
    }
    out
}

/// `{"Event": [{"matcher": "...", "hooks": [{"type": "command", "command": "..."}]}]}` → one node per command.
pub fn hooks(s: &mut Scan, hooks: Option<&Value>, scope: Scope, source: &Path, parent: Option<(&str, &Path)>) {
    let Some(events) = hooks.and_then(Value::as_object) else { return };
    for (event, groups) in events {
        for (i, group) in groups.as_array().into_iter().flatten().enumerate() {
            for (j, h) in group["hooks"].as_array().into_iter().flatten().enumerate() {
                let mut found = vec![];
                let command = h
                    .get("command")
                    .and_then(Value::as_str)
                    .map(|c| secrets::mask_command(c, &mut found));
                let n = s.push(Kind::Hook, scope, source, event);
                n.id = format!("{}/{i}/{j}", n.id);
                n.meta = json!({
                    "event": event,
                    "matcher": group.get("matcher"),
                    "type": h.get("type"),
                    "command": command,
                    "plaintext_secrets": found,
                    // Position in the file, to find it again when moving it.
                    "index": [i, j],
                });
                if let Some((id, root)) = parent {
                    n.parent = Some(id.to_string());
                    n.meta["plugin_root"] = json!(root);
                }
            }
        }
    }
}
