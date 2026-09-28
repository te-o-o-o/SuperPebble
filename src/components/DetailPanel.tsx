import { useState } from "react";
import { KIND_LABEL, fmtTokens, relTime, scopeOf } from "../theme";
import type { Graph, PNode } from "../types";
import { home } from "./home";
import { ItemList } from "./ItemList";
import { ScopeBadge } from "./ScopeBadge";

export function DetailPanel({ graph, node, onOpen, onSelect }: { graph: Graph; node?: PNode; onOpen: (path: string) => void; onSelect: (id: string) => void }) {
  if (!node) {
    return (
      <aside className="detail empty">
        <p className="muted">Sélectionne un galet pour voir son détail.</p>
      </aside>
    );
  }
  const issues = graph.issues.filter((i) => i.nodes.includes(node.id));
  const others = (ids: string[]) => graph.nodes.filter((n) => ids.includes(n.id) && n.id !== node.id);
  const m = node.meta;

  return (
    <aside className="detail">
      <div className="muted kind">
        <ScopeBadge scope={node.scope} /> {KIND_LABEL[node.kind]} · scope {scopeOf(node.scope).label}
      </div>
      <h2>{node.name}</h2>
      <code className="path">{home(node.source)}</code>

      <div className="stats">
        <div>
          <span className="muted">Au démarrage</span>
          <strong className="mono">{node.tokens == null ? "inconnu" : `~${fmtTokens(node.tokens)} tok`}</strong>
        </div>
        <div>
          <span className="muted">Modifié</span>
          <strong>{node.modified ? relTime(node.modified) : "—"}</strong>
        </div>
      </div>

      {issues.map((i, k) => (
        <div key={k} className={`card ${i.severity}`}>
          <strong>{i.message}</strong>
          {others(i.nodes).map((o) => (
            <p key={o.id} className="muted">
              Aussi dans le scope {scopeOf(o.scope).label} : <code>{home(o.source)}</code>
            </p>
          ))}
        </div>
      ))}

      {m.frontmatter && <Section title="Frontmatter"><LongText key={node.id} text={m.frontmatter} /></Section>}
      {node.kind === "mcp" && (
        <Section title="Serveur">
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
        <Section title="Contenu">
          <ItemList graph={graph} items={graph.nodes.filter((n) => n.parent === node.id).sort((a, b) => a.kind.localeCompare(b.kind) || a.name.localeCompare(b.name))} showKind onSelect={onSelect} />
        </Section>
      )}
      {m.imports?.length > 0 && (
        <Section title="Imports">
          <pre>{m.imports.map(home).join("\n")}</pre>
        </Section>
      )}

      <label className="toggle" title="Lecture seule jusqu'au ménage (M2)">
        <input type="checkbox" checked={node.enabled} disabled readOnly /> Actif
        <span className="muted">lecture seule</span>
      </label>

      <span className="spacer" />
      <div className="actions">
        <button onClick={() => onOpen(node.source)}>Ouvrir dans l'éditeur</button>
        <button className="danger" disabled title="Arrive en M2">
          Supprimer…
        </button>
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
          {open ? "Réduire" : "Voir tout"}
        </button>
      )}
    </>
  );
}
