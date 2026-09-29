//! Cross-node checks. Nothing is executed: we only look for files and binaries on disk.

use crate::model::{Issue, Kind, Node, Severity};
use crate::scan::home;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

pub fn check(nodes: &[Node]) -> Vec<Issue> {
    let mut out = vec![];
    for n in nodes.iter().filter(|n| n.enabled) {
        let issue = |rule, severity, message| Issue {
            rule,
            severity,
            nodes: vec![n.id.clone()],
            message,
        };
        match n.kind {
            Kind::Mcp => {
                if let Some(cmd) = n.meta["command"].as_str() {
                    if !found(&expand(cmd, n)) {
                        out.push(issue(
                            "orphan-mcp",
                            Severity::Error,
                            format!("MCP orphelin : {} (commande introuvable)", n.name),
                        ));
                    }
                }
                for key in n.meta["plaintext_secrets"].as_array().into_iter().flatten() {
                    out.push(issue(
                        "plaintext-secret",
                        Severity::Warning,
                        format!("Secret en clair : {} ({})", key.as_str().unwrap_or(""), n.name),
                    ));
                }
            }
            Kind::Hook => {
                if let Some(missing) = n.meta["command"].as_str().and_then(|c| missing_in_command(c, n)) {
                    out.push(issue(
                        "broken-hook",
                        Severity::Error,
                        format!("Hook cassé : {} ({missing} introuvable)", n.name),
                    ));
                }
            }
            _ => {}
        }
    }
    out.extend(duplicates(nodes));
    out
}

/// Same kind + name loaded from several scopes. Plugin items are namespaced, so they never clash.
fn duplicates(nodes: &[Node]) -> Vec<Issue> {
    let mut groups: BTreeMap<(String, &str), Vec<&Node>> = BTreeMap::new();
    for n in nodes.iter().filter(|n| n.enabled && n.parent.is_none()) {
        if matches!(n.kind, Kind::Skill | Kind::Agent | Kind::Command | Kind::Mcp) {
            groups.entry((format!("{:?}", n.kind), &n.name)).or_default().push(n);
        }
    }
    groups
        .into_values()
        .filter(|g| g.len() > 1)
        .map(|g| Issue {
            rule: "duplicate",
            severity: Severity::Warning,
            nodes: g.iter().map(|n| n.id.clone()).collect(),
            message: format!("Doublon : {}", g[0].name),
        })
        .collect()
}

/// First word must be a known binary; any absolute path argument must exist.
fn missing_in_command(cmd: &str, n: &Node) -> Option<String> {
    let words = split(&expand(cmd, n));
    let first = words.first()?;
    if !found(first) {
        return Some(first.clone());
    }
    words[1..]
        .iter()
        .find(|w| w.starts_with('/') && !w.contains('$') && !Path::new(w).exists())
        .cloned()
}

fn expand(s: &str, n: &Node) -> String {
    let mut s = s.replace("$HOME", &home().to_string_lossy());
    if let Some(root) = n.meta["plugin_root"].as_str() {
        s = s.replace("${CLAUDE_PLUGIN_ROOT}", root);
    }
    if let Some(rest) = s.strip_prefix("~/") {
        s = home().join(rest).to_string_lossy().into_owned();
    }
    s
}

static SEARCH_PATH: OnceLock<String> = OnceLock::new();

/// Overrides `$PATH` for binary lookups, e.g. with the user's login-shell PATH when the app
/// was launched from a GUI. Safe to call from another thread, unlike `env::set_var`.
pub fn set_search_path(path: String) {
    let _ = SEARCH_PATH.set(path);
}

/// Is `cmd` an existing path, or a file somewhere in the search path?
fn found(cmd: &str) -> bool {
    if cmd.contains('/') || cmd.contains('\\') {
        return Path::new(cmd).exists();
    }
    let path = SEARCH_PATH.get().map(Into::into).or_else(|| std::env::var_os("PATH"));
    // Windows resolves `npx` to `npx.cmd` etc. through PATHEXT.
    let exts: Vec<String> = if cfg!(windows) {
        let pathext = std::env::var("PATHEXT").unwrap_or_else(|_| ".EXE;.CMD;.BAT;.COM".into());
        std::iter::once(String::new())
            .chain(pathext.split(';').map(str::to_lowercase))
            .collect()
    } else {
        vec![String::new()]
    };
    path.is_some_and(|p| std::env::split_paths(&p).any(|d: PathBuf| exts.iter().any(|e| d.join(format!("{cmd}{e}")).is_file())))
}

/// Minimal shell split (quotes only, no escapes/globs), enough to find script paths.
fn split(cmd: &str) -> Vec<String> {
    let (mut out, mut cur, mut quote) = (vec![], String::new(), None);
    for c in cmd.chars() {
        match (quote, c) {
            (Some(q), c) if c == q => quote = None,
            (None, '"' | '\'') => quote = Some(c),
            (None, c) if c.is_whitespace() => {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
            }
            _ => cur.push(c),
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Scope;
    use serde_json::json;

    fn node(kind: Kind, name: &str, scope: Scope, meta: serde_json::Value) -> Node {
        Node {
            id: format!("{name}{scope:?}"),
            kind,
            name: name.into(),
            scope,
            source: "/x".into(),
            enabled: true,
            parent: None,
            tokens: None,
            modified: None,
            meta,
        }
    }

    #[test]
    fn split_handles_quotes() {
        assert_eq!(split(r#"bash '/a b/c.sh' session"#), ["bash", "/a b/c.sh", "session"]);
    }

    #[test]
    fn rules() {
        let nodes = [
            node(Kind::Mcp, "gone", Scope::User, json!({"command": "definitely-not-a-binary-xyz"})),
            node(Kind::Mcp, "ok", Scope::User, json!({"command": "sh"})),
            node(Kind::Hook, "Stop", Scope::User, json!({"command": "sh /nope/notify.sh"})),
            node(Kind::Skill, "dup", Scope::User, json!({})),
            node(Kind::Skill, "dup", Scope::Project, json!({})),
        ];
        let rules: Vec<_> = check(&nodes).iter().map(|i| i.rule).collect();
        assert_eq!(rules, ["orphan-mcp", "broken-hook", "duplicate"]);
    }
}
