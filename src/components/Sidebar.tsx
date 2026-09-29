import { t } from "../i18n";
import { GROUPS, SCOPES, groupOf, type GroupKey } from "../theme";
import type { Graph, Scope } from "../types";
import { home } from "./home";
import { ScopeBadge } from "./ScopeBadge";

type Props = {
  graph?: Graph;
  hidden: Set<GroupKey>;
  onToggle: (k: GroupKey) => void;
  scope: Scope | null;
  onScope: (s: Scope | null) => void;
  onAccounts: () => void;
};

export function Sidebar({ graph, hidden, onToggle, scope, onScope, onAccounts }: Props) {
  const top = graph?.nodes.filter((n) => !n.parent) ?? [];
  const count = (k: GroupKey) => top.filter((n) => groupOf(n.kind).key === k && (!scope || n.scope === scope)).length;
  const scopeCount = (sc: Scope) => top.filter((n) => n.scope === sc).length;
  return (
    <aside className="sidebar">
      <h3>Harness</h3>
      <div className="harness active">
        <span className="dot" /> Claude Code
      </div>
      {["Claude Desktop", "Codex", "Cursor"].map((h) => (
        <div key={h} className="harness off">
          <span className="dot" /> {h} <span className="muted">{t("soon")}</span>
        </div>
      ))}

      <h3>{t("Show")}</h3>
      {GROUPS.map((g) => (
        <label key={g.key} className="filter">
          <input type="checkbox" checked={!hidden.has(g.key)} onChange={() => onToggle(g.key)} />
          <span className="ring" style={{ borderColor: g.color }} />
          {g.sidebar}
          <span className="muted">{count(g.key)}</span>
        </label>
      ))}

      <h3>Scope</h3>
      {SCOPES.filter((sc) => sc.key !== "managed" || scopeCount(sc.key)).map((sc) => (
        <button key={sc.key} className={`scope-filter${scope === sc.key ? " active" : ""}`} onClick={() => onScope(scope === sc.key ? null : sc.key)}>
          <ScopeBadge scope={sc.key} />
          {sc.label}
          <code>{sc.key === "user" ? home(graph?.config_dir ?? sc.where) : sc.where}</code>
          <span className="muted">{scopeCount(sc.key)}</span>
        </button>
      ))}
      <div className="legend">
        <span className="scope dashed" /> {t("broken / orphan")}
      </div>

      <span className="spacer" />
      <button onClick={onAccounts}>{t("Manage accounts")}</button>
    </aside>
  );
}
