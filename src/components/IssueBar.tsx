import { COLORS } from "../theme";
import type { Issue } from "../types";

export function IssueBar({ issues, onSelect }: { issues: Issue[]; onSelect: (id: string) => void }) {
  return (
    <footer className="issuebar">
      <strong>{issues.length ? `${issues.length} point${issues.length > 1 ? "s" : ""} à revoir` : "Rien à signaler"}</strong>
      <div className="issues">
        {issues.map((i, k) => (
          <button key={k} className="issue" onClick={() => onSelect(i.nodes[0])}>
            <span className="dot" style={{ background: i.severity === "error" ? COLORS.error : COLORS.warning }} />
            {i.message}
          </button>
        ))}
      </div>
      <button className="primary" disabled title="Arrive en M2">
        Faire le ménage
      </button>
    </footer>
  );
}
