import { issueText, t } from "../i18n";
import { COLORS } from "../theme";
import type { Issue } from "../types";

export function IssueBar({ issues, onSelect }: { issues: Issue[]; onSelect: (id: string) => void }) {
  return (
    <footer className="issuebar">
      <strong>{issues.length > 1 ? t("{0} issues to review", issues.length) : issues.length ? t("1 issue to review") : t("Nothing to report")}</strong>
      <div className="issues">
        {issues.map((i, k) => (
          <button key={k} className="issue" onClick={() => onSelect(i.nodes[0])}>
            <span className="dot" style={{ background: i.severity === "error" ? COLORS.error : COLORS.warning }} />
            {issueText(i)}
          </button>
        ))}
      </div>
      <button className="primary" disabled title={t("Comes in M2")}>
        {t("Clean up")}
      </button>
    </footer>
  );
}
