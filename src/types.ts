// Mirrors crates/superpebble/src/model.rs.
export type Kind = "plugin" | "skill" | "mcp" | "agent" | "command" | "hook" | "config";
export type Scope = "managed" | "user" | "project" | "local";
export type Severity = "info" | "warning" | "error";

export interface PNode {
  id: string;
  kind: Kind;
  name: string;
  scope: Scope;
  source: string;
  enabled: boolean;
  parent: string | null;
  tokens: number | null;
  modified: number | null;
  meta: Record<string, any>;
}

export interface Issue {
  rule: string;
  severity: Severity;
  nodes: string[];
  args: string[];
  message: string;
}

export interface Graph {
  config_dir: string;
  project: string | null;
  scanned_at: number;
  nodes: PNode[];
  issues: Issue[];
  budget: Budget;
}

export interface Budget {
  total: number;
  claude_md: number;
  skills: number;
  agents: number;
  mcp_servers: number;
  top: string[];
}

export interface Account {
  name: string;
  config_dir: string;
  is_default: boolean;
  email: string | null;
  shared: { item: string; link: string | null; own: boolean }[];
  alias: { kind: "none" } | { kind: "managed" } | { kind: "manual"; line: number };
  /** Lives in a WSL distro: read-only for now. */
  wsl: boolean;
}
