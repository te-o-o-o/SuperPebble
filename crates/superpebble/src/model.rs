use serde::Serialize;
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

#[derive(Serialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
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

#[derive(Serialize, Clone, Copy, PartialEq, Eq, Debug)]
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
    pub message: String,
}

#[derive(Serialize, Debug)]
pub struct Graph {
    pub config_dir: PathBuf,
    pub project: Option<PathBuf>,
    pub scanned_at: u64,
    pub nodes: Vec<Node>,
    pub issues: Vec<Issue>,
}
