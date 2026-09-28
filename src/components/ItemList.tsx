import { KIND_LABEL, fmtTokens, issuesByNode } from "../theme";
import type { Graph, PNode } from "../types";
import { ScopeBadge } from "./ScopeBadge";

/** Clickable list of pebbles, used for a branch's or a plugin's content. */
export function ItemList({ graph, items, showKind, onSelect }: { graph: Graph; items: PNode[]; showKind?: boolean; onSelect: (id: string) => void }) {
  const issues = issuesByNode(graph.issues);
  return (
    <div className="branch-list">
      {items.map((n) => (
        <button key={n.id} onClick={() => onSelect(n.id)} className={n.enabled ? undefined : "off"}>
          <ScopeBadge scope={n.scope} />
          {n.name}
          {showKind && <span className="muted kind-tag">{KIND_LABEL[n.kind]}</span>}
          {issues.has(n.id) && <span className="dot" />}
          <span className="muted mono">{n.tokens ? fmtTokens(n.tokens) : ""}</span>
        </button>
      ))}
    </div>
  );
}
