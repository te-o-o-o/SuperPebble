import { useEffect, useRef, useState } from "react";
import { api } from "../api";
import { t } from "../i18n";
import { scopeOf } from "../theme";
import type { Account, Kind, PNode, Scope } from "../types";
import { ScopeBadge } from "./ScopeBadge";

/** Where each kind can live. Mirrors crates/superpebble/src/transfer.rs. */
const TARGETS: Partial<Record<Kind, Scope[]>> = {
  skill: ["user", "project"],
  agent: ["user", "project"],
  command: ["user", "project"],
  mcp: ["user", "local", "project"],
  hook: ["user", "project", "local"],
  plugin: ["user", "project", "local"],
};

export const movable = (n: PNode) => !!TARGETS[n.kind] && !n.parent && n.scope !== "managed";

type Props = {
  node: PNode | null;
  accounts: Account[];
  account: string;
  project: string | null;
  onClose: () => void;
  onDone: () => void;
};

export function TransferDialog({ node, accounts, account, project, onClose, onDone }: Props) {
  const ref = useRef<HTMLDialogElement>(null);
  const [to, setTo] = useState(account);
  const [scope, setScope] = useState<Scope>();
  const [copy, setCopy] = useState(false);
  const [confirmed, setConfirmed] = useState(false);
  const [error, setError] = useState<string>();

  useEffect(() => {
    if (node) ref.current?.showModal();
    else ref.current?.close();
    setTo(account);
    setScope(undefined);
    setCopy(false);
    setConfirmed(false);
    setError(undefined);
  }, [node, account]);

  if (!node) return <dialog ref={ref} className="accounts" />;
  // A plugin changes account only from and to the user scope: project installs belong to the project.
  const plugin = node.kind === "plugin";
  const otherAccount = to !== account;
  const scopes: Scope[] = plugin && otherAccount ? ["user"] : (TARGETS[node.kind] ?? []).filter((s) => s === "user" || project);
  const secret = node.kind === "mcp" && node.meta.plaintext_secrets?.length > 0 && scope === "project";
  const ready = scope && scopes.includes(scope) && !(!otherAccount && scope === node.scope) && (!secret || confirmed);

  const submit = () =>
    api.transfer(account, project, node.id, to, scope!, copy).then(
      () => {
        onDone();
        onClose();
      },
      (e) => setError(t(String(e))),
    );

  return (
    <dialog ref={ref} className="accounts" onClose={onClose} onClick={(e) => e.target === ref.current && onClose()}>
      <header>
        <h2>{t("Move or copy {0}", node.name)}</h2>
        <button className="ghost" onClick={onClose} aria-label={t("Close")}>
          ✕
        </button>
      </header>

      <section className="account">
        <div className="row">
          <span className="muted small label">{t("Account")}</span>
          <label className="select">
            <select value={to} disabled={plugin && node.scope !== "user"} onChange={(e) => setTo(e.target.value)}>
              {accounts.filter((a) => !a.wsl).map((a) => (
                <option key={a.config_dir} value={a.config_dir}>
                  {a.name}
                </option>
              ))}
            </select>
          </label>
        </div>
        <div className="row">
          <span className="muted small label">{t("Scope")}</span>
          {scopes.map((s) => (
            <button key={s} className={`chip-toggle${scope === s ? " on" : ""}`} onClick={() => setScope(s)}>
              <ScopeBadge scope={s} /> {scopeOf(s).label}
            </button>
          ))}
        </div>
        <div className="row">
          <span className="muted small label">{t("Mode")}</span>
          <button className={`chip-toggle${copy ? "" : " on"}`} onClick={() => setCopy(false)}>
            {t("Move")}
          </button>
          <button className={`chip-toggle${copy ? " on" : ""}`} onClick={() => setCopy(true)}>
            {t("Copy")}
          </button>
        </div>
        {plugin && otherAccount && <p className="muted small">{t("Claude Code installs it on that account itself: this needs the network.")}</p>}
        {(node.kind === "mcp" || node.kind === "hook" || plugin) && (
          <p className="muted small">{t("Close your Claude Code sessions first: they rewrite these files too.")}</p>
        )}
        {secret && (
          <label className="switch warn">
            <input type="checkbox" checked={confirmed} onChange={(e) => setConfirmed(e.target.checked)} />
            {t("It holds a plaintext secret and .mcp.json is often committed. Move it anyway.")}
          </label>
        )}
        <button className="primary" disabled={!ready} onClick={submit}>
          {copy ? t("Copy") : t("Move")}
        </button>
      </section>
      {error && <p className="error">{error}</p>}
    </dialog>
  );
}
