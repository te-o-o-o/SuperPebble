import { issueText, t } from "../i18n";
import { COLORS } from "../theme";
import type { Issue } from "../types";

type Props = { issues: Issue[]; onSelect: (id: string) => void; onCleanup?: () => void };

export function IssueBar({ issues, onSelect, onCleanup }: Props) {
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
      <button className="primary" disabled={!onCleanup || !issues.some((i) => i.fix)} onClick={onCleanup}>
        {t("Clean up")}
      </button>
    </footer>
  );
}
