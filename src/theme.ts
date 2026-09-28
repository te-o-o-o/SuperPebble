import type { Issue, Kind, PNode, Scope } from "./types";

export type GroupKey = "plugin" | "skill" | "mcp" | "hook" | "agent" | "config";

/** Branches around Claude Code. `angle` in degrees, 0 = right, clockwise. */
export const GROUPS: { key: GroupKey; label: string; sidebar: string; kinds: Kind[]; color: string; angle: number }[] = [
  { key: "plugin", label: "Plugins", sidebar: "Plugins", kinds: ["plugin"], color: "#d9774b", angle: -90 },
  { key: "skill", label: "Skills", sidebar: "Skills", kinds: ["skill"], color: "#7aa7e0", angle: -22 },
  { key: "mcp", label: "MCP", sidebar: "MCP servers", kinds: ["mcp"], color: "#5fbfb0", angle: 38 },
  { key: "hook", label: "Hooks", sidebar: "Hooks", kinds: ["hook"], color: "#9cc27a", angle: 90 },
  { key: "agent", label: "Agents", sidebar: "Agents & commands", kinds: ["agent", "command"], color: "#d8c07a", angle: 148 },
  { key: "config", label: "Conf", sidebar: "Fichiers de conf", kinds: ["config"], color: "#b9a3e8", angle: 208 },
];

export const groupOf = (kind: Kind) => GROUPS.find((g) => g.kinds.includes(kind))!;

export const COLORS = { center: "#d9774b", error: "#e0645a", warning: "#e0a24a", text: "#e8e4dc" };

/** Scopes, by increasing precedence. Filled badges, so they read apart from the kind-coloured outlines. */
export const SCOPES: { key: Scope; letter: string; label: string; where: string; color: string }[] = [
  { key: "user", letter: "u", label: "user", where: "~/.claude", color: "#d8d3c8" },
  { key: "project", letter: "p", label: "projet", where: ".claude/", color: "#f08bb4" },
  { key: "local", letter: "l", label: "local", where: "*.local.*", color: "#ffd166" },
  { key: "managed", letter: "m", label: "managed", where: "/etc, /Library", color: "#8fb3ff" },
];
export const scopeOf = (s: Scope) => SCOPES.find((x) => x.key === s)!;

export const KIND_LABEL: Record<Kind, string> = {
  plugin: "Plugin",
  skill: "Skill",
  mcp: "MCP server",
  agent: "Agent",
  command: "Command",
  hook: "Hook",
  config: "Fichier de conf",
};

/** Short label drawn on the pebble for the worst issue of a node. */
export const RULE_CHIP: Record<string, string> = {
  "orphan-mcp": "introuvable",
  "broken-hook": "cassé",
  "plaintext-secret": "secret",
  duplicate: "doublon",
  "invalid-skill": "invalide",
  "invalid-json": "illisible",
};

export function seed(id: string) {
  let h = 2166136261;
  for (let i = 0; i < id.length; i++) h = Math.imul(h ^ id.charCodeAt(i), 16777619);
  return (h >>> 0) % 2 ** 31 || 1;
}

export const fmtTokens = (t: number) => (t >= 1000 ? `${(t / 1000).toFixed(1)}k` : `${t}`);

const rtf = new Intl.RelativeTimeFormat("fr", { numeric: "auto" });
export function relTime(unixSecs: number) {
  const s = unixSecs - Date.now() / 1000;
  const units: [Intl.RelativeTimeFormatUnit, number][] = [["year", 31536000], ["month", 2592000], ["day", 86400], ["hour", 3600], ["minute", 60]];
  for (const [u, n] of units) if (Math.abs(s) >= n) return rtf.format(Math.round(s / n), u);
  return "à l'instant";
}

export const basename = (p: string) => p.replace(/["']/g, "").split("/").pop() ?? p;

/** Right-hand hint on an item pebble. */
export function hint(n: PNode, childCount: number) {
  switch (n.kind) {
    case "plugin":
      return `${childCount}`;
    case "agent":
    case "command":
      return n.kind;
    case "hook": {
      const words: string[] = (n.meta.command ?? "").split(/\s+/);
      return basename(words.find((w) => w.includes("/")) ?? words[0] ?? "");
    }
    case "config":
      return n.tokens ? fmtTokens(n.tokens) : "";
    case "mcp":
      return n.meta.type === "stdio" ? "" : n.meta.type;
    default:
      return "";
  }
}

export function issuesByNode(issues: Issue[]) {
  const m = new Map<string, Issue[]>();
  for (const i of issues) for (const id of i.nodes) m.set(id, [...(m.get(id) ?? []), i]);
  return m;
}
