import { useState } from "react";
import { issueText, t } from "../i18n";
import { KIND_LABEL, fmtTokens, relTime, scopeOf } from "../theme";
import type { Graph, PNode } from "../types";
import { home } from "./home";
import { ItemList } from "./ItemList";
import { ScopeBadge } from "./ScopeBadge";
import { movable } from "./TransferDialog";

type Props = { graph: Graph; node?: PNode; onOpen: (path: string) => void; onSelect: (id: string) => void; onMove?: (n: PNode) => void };

export function DetailPanel({ graph, node, onOpen, onSelect, onMove }: Props) {
  if (!node) {
    return (
      <aside className="detail empty">
        <p className="muted">{t("Select a pebble to see its details.")}</p>
      </aside>
    );
  }
  const issues = graph.issues.filter((i) => i.nodes.includes(node.id));
  const others = (ids: string[]) => graph.nodes.filter((n) => ids.includes(n.id) && n.id !== node.id);
  const m = node.meta;

  return (
    <aside className="detail">
      <div className="muted kind">
        <ScopeBadge scope={node.scope} /> {t("{0} · scope {1}", KIND_LABEL[node.kind], scopeOf(node.scope).label)}
      </div>
      <h2>{node.name}</h2>
      <code className="path">{home(node.source)}</code>

      <div className="stats">
        <div>
          <span className="muted">{t("At startup")}</span>
          <strong className="mono">{node.tokens == null ? t("unknown") : `~${fmtTokens(node.tokens)} tok`}</strong>
        </div>
        <div>
          <span className="muted">{t("Modified")}</span>
          <strong>{node.modified ? relTime(node.modified) : "—"}</strong>
        </div>
      </div>

      {issues.map((i, k) => (
        <div key={k} className={`card ${i.severity}`}>
          <strong>{issueText(i)}</strong>
          {others(i.nodes).map((o) => (
            <p key={o.id} className="muted">
              {t("Also in scope {0}:", scopeOf(o.scope).label)} <code>{home(o.source)}</code>
            </p>
          ))}
        </div>
      ))}

      {m.frontmatter && <Section title="Frontmatter"><LongText key={node.id} text={m.frontmatter} /></Section>}
      {node.kind === "mcp" && (
        <Section title={t("Server")}>
          <pre>
            {[
              `type: ${m.type}`,
              m.command && `command: ${m.command} ${(m.args ?? []).join(" ")}`,
              m.url && `url: ${m.url}`,
              m.env_keys?.length && `env: ${m.env_keys.join(", ")}`,
              m.header_keys?.length && `headers: ${m.header_keys.join(", ")}`,
            ]
              .filter(Boolean)
              .join("\n")}
          </pre>
        </Section>
      )}
      {node.kind === "hook" && (
        <Section title="Hook">
          <pre>{[`event: ${m.event}`, m.matcher && `matcher: ${m.matcher}`, `command: ${m.command}`].filter(Boolean).join("\n")}</pre>
        </Section>
      )}
      {node.kind === "plugin" && (
        <Section title="Plugin">
          <pre>{[`id: ${m.id}`, `version: ${m.version}`, m.description && `\n${m.description}`].filter(Boolean).join("\n")}</pre>
        </Section>
      )}
      {node.kind === "plugin" && (
        <Section title={t("Content")}>
          <ItemList graph={graph} items={graph.nodes.filter((n) => n.parent === node.id).sort((a, b) => a.kind.localeCompare(b.kind) || a.name.localeCompare(b.name))} showKind onSelect={onSelect} />
        </Section>
      )}
      {m.imports?.length > 0 && (
        <Section title="Imports">
          <pre>{m.imports.map(home).join("\n")}</pre>
        </Section>
      )}

      <label className="toggle" title={t("Read-only until the cleanup (M2)")}>
        <input type="checkbox" checked={node.enabled} disabled readOnly /> {t("Enabled")}
        <span className="muted">{t("read-only")}</span>
      </label>

      <span className="spacer" />
      <div className="actions">
        <button onClick={() => onOpen(node.source)}>{t("Open in editor")}</button>
        {onMove && movable(node) && <button onClick={() => onMove(node)}>{t("Move…")}</button>}
      </div>
    </aside>
  );
}

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <section>
      <h3>{title}</h3>
      {children}
    </section>
  );
}

/** Long text starts clamped with a fade; the key on the caller resets it per node. */
function LongText({ text }: { text: string }) {
  const [open, setOpen] = useState(false);
  const long = text.length > 400 || text.split("\n").length > 10;
  return (
    <>
      <pre className={long && !open ? "clamped" : undefined}>{text}</pre>
      {long && (
        <button className="link" onClick={() => setOpen(!open)}>
          {open ? t("Collapse") : t("Show all")}
        </button>
      )}
    </>
  );
}
