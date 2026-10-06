import { t } from "../i18n";
import { COLORS, GROUPS, fmtTokens, type GroupKey } from "../theme";
import type { Graph } from "../types";
import { ItemList } from "./ItemList";

const color = (k: GroupKey) => GROUPS.find((g) => g.key === k)!.color;

/** What Claude Code loads at startup, opened from the center pebble. */
export function BudgetPanel({ graph, onSelect }: { graph: Graph; onSelect: (id: string) => void }) {
  const b = graph.budget;
  const rows: [string, number, string][] = [
    ["CLAUDE.md", b.claude_md, color("config")],
    [t("Skills & commands"), b.skills, color("skill")],
    ["Agents", b.agents, color("agent")],
  ];
  const top = b.top.flatMap((id) => graph.nodes.find((n) => n.id === id) ?? []);

  return (
    <aside className="detail">
      <div className="muted kind">
        <span className="ring" style={{ borderColor: COLORS.center }} /> {t("Context budget")}
      </div>
      <h2 className="mono">~{fmtTokens(b.total)} tok</h2>
      <span className="muted">{t("loaded at startup, estimated")}</span>

      <section className="budget">
        {rows.map(([label, tokens, c]) => (
          <div key={label}>
            <span>{label}</span>
            <span className="muted mono">~{fmtTokens(tokens)}</span>
            <span className="bar" style={{ width: `${b.total ? (100 * tokens) / b.total : 0}%`, background: c }} />
          </div>
        ))}
        <div>
          <span>MCP</span>
          <span className="muted">{t("{0} servers, not counted", b.mcp_servers)}</span>
        </div>
      </section>

      <section>
        <h3>{t("Heaviest")}</h3>
        <ItemList graph={graph} items={top} showKind onSelect={onSelect} />
      </section>

      <p className="muted note">
        {t(
          "Estimate: characters ÷ 4 of CLAUDE.md files with their @imports, and of the name and description of each skill, command and agent. MCP tool definitions are not counted: reading them would mean starting the servers.",
        )}
      </p>
    </aside>
  );
}
