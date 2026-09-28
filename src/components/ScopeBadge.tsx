import { scopeOf } from "../theme";
import type { Scope } from "../types";

export function ScopeBadge({ scope }: { scope: Scope }) {
  const s = scopeOf(scope);
  return (
    <span className="scope" style={{ background: s.color }} title={`scope ${s.label}`}>
      {s.letter}
    </span>
  );
}
