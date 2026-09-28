import { GROUPS, fmtTokens, type GroupKey } from "../theme";
import type { Graph } from "../types";
import { ItemList } from "./ItemList";

/** Detail panel for a whole branch (a group pebble around Claude Code). */
export function BranchPanel({ graph, group, onSelect }: { graph: Graph; group: GroupKey; onSelect: (id: string) => void }) {
  const g = GROUPS.find((x) => x.key === group)!;
  const items = graph.nodes.filter((n) => !n.parent && g.kinds.includes(n.kind)).sort((a, b) => a.name.localeCompare(b.name));
  const ids = new Set(items.map((n) => n.id));
  const branchIssues = graph.issues.filter((i) => i.nodes.some((id) => ids.has(id)));
  const tokens = items.filter((n) => n.enabled).reduce((s, n) => s + (n.tokens ?? 0), 0);

  return (
    <aside className="detail">
      <div className="muted kind">
        <span className="ring" style={{ borderColor: g.color }} /> Branche
      </div>
      <h2>{g.sidebar}</h2>

      <div className="stats">
        <div>
          <span className="muted">Éléments</span>
          <strong>{items.length}</strong>
        </div>
        <div>
          <span className="muted">Au démarrage</span>
          <strong className="mono">~{fmtTokens(tokens)} tok</strong>
        </div>
      </div>

      {branchIssues.map((i, k) => (
        <div key={k} className={`card ${i.severity}`}>
          <strong>{i.message}</strong>
        </div>
      ))}

      <section>
        <h3>Contenu</h3>
        <ItemList graph={graph} items={items} onSelect={onSelect} />
      </section>
    </aside>
  );
}
