use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;

#[derive(Serialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Plugin,
    Skill,
    Mcp,
    Agent,
    Command,
    Hook,
    Config,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Scope {
    Managed,
    User,
    Project,
    Local,
}

#[derive(Serialize, Clone, Debug)]
pub struct Node {
    /// Stable across scans: `kind:scope:source#name`. The UI derives its drawing seed from it.
    pub id: String,
    pub kind: Kind,
    pub name: String,
    pub scope: Scope,
    /// File that declares the element.
    pub source: PathBuf,
    pub enabled: bool,
    /// Plugin that ships this element, if any.
    pub parent: Option<String>,
    /// chars/4 of what Claude Code loads at startup. None = unknown (MCP tools, hooks).
    pub tokens: Option<u32>,
    /// Unix seconds.
    pub modified: Option<u64>,
    /// Kind-specific data: frontmatter, MCP command, hook event... Never holds secret values.
    pub meta: Value,
}

#[derive(Serialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Warning,
    Error,
}

#[derive(Serialize, Clone, Debug)]
pub struct Issue {
    pub rule: &'static str,
    pub severity: Severity,
    pub nodes: Vec<String>,
    /// What fills the rule's message; the UI renders them in the user's language.
    pub args: Vec<String>,
    /// English message, for the CLI and as a fallback.
    pub message: String,
}

impl Issue {
    pub fn new(rule: &'static str, severity: Severity, nodes: Vec<String>, args: Vec<String>) -> Self {
        let a = |i: usize| args.get(i).map_or("", String::as_str);
        let message = match rule {
            "orphan-mcp" => format!("Orphan MCP: {} (command not found)", a(0)),
            "plaintext-secret" => format!("Plaintext secret: {} ({})", a(0), a(1)),
            "broken-hook" => format!("Broken hook: {} ({} not found)", a(0), a(1)),
            "duplicate" => format!("Duplicate: {}", a(0)),
            "invalid-skill" => format!("Invalid {}: {} ({})", a(0), a(1), a(2)),
            "invalid-json" => format!("Unreadable JSON: {} ({})", a(0), a(1)),
            _ => rule.to_string(),
        };
        Issue {
            rule,
            severity,
            nodes,
            args,
            message,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct Graph {
    pub config_dir: PathBuf,
    pub project: Option<PathBuf>,
    pub scanned_at: u64,
    pub nodes: Vec<Node>,
    pub issues: Vec<Issue>,
}
