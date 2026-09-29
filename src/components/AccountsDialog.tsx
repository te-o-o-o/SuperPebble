import { useEffect, useRef, useState } from "react";
import { api } from "../api";
import { lang, t } from "../i18n";
import type { Account } from "../types";
import { home } from "./home";

const SHAREABLE = ["skills", "agents", "commands", "CLAUDE.md"];
const shellAliases = !navigator.userAgent.includes("Windows");

type Props = { open: boolean; accounts: Account[]; onClose: () => void; onChange: (a: Account[]) => void };

export function AccountsDialog({ open, accounts, onClose, onChange }: Props) {
  const ref = useRef<HTMLDialogElement>(null);
  const [error, setError] = useState<string>();
  const [done, setDone] = useState<string>();

  useEffect(() => {
    if (open) ref.current?.showModal();
    else ref.current?.close();
    setError(undefined);
    setDone(undefined);
  }, [open]);

  /** Runs a write, then shows the fresh list or the backend's message. */
  const run = (p: Promise<Account[]>, success?: string) =>
    p.then(
      (a) => {
        onChange(a);
        setError(undefined);
        setDone(success);
      },
      (e) => setError(t(String(e))),
    );

  const source = accounts.find((a) => a.is_default);
  return (
    <dialog ref={ref} className="accounts" onClose={onClose} onClick={(e) => e.target === ref.current && onClose()}>
      <header>
        <h2>{t("Accounts")}</h2>
        <button className="ghost" onClick={onClose} aria-label={t("Close")}>
          ✕
        </button>
      </header>
      {lang === "fr" ? (
        <p className="muted intro">
          Un compte est un dossier <code>CLAUDE_CONFIG_DIR</code>. Partager un élément crée un lien symbolique vers{" "}
          <code>{home(source?.config_dir ?? "~/.claude")}</code>. MCP, plugins et identifiants restent propres à chaque compte.
        </p>
      ) : (
        <p className="muted intro">
          An account is a <code>CLAUDE_CONFIG_DIR</code> folder. Sharing an item creates a symlink to{" "}
          <code>{home(source?.config_dir ?? "~/.claude")}</code>. MCP servers, plugins and credentials stay per account.
        </p>
      )}

      {accounts.map((a) => (
        <AccountCard key={a.config_dir} a={a} run={run} />
      ))}

      <NewAccount run={run} />
      {done && <p className="done">{done}</p>}
      {error && <p className="error">{error}</p>}
    </dialog>
  );
}

function AccountCard({ a, run }: { a: Account; run: (p: Promise<Account[]>) => void }) {
  return (
    <section className="account">
      <div className="account-head">
        <strong>{a.name}</strong>
        <code className="muted">{home(a.config_dir)}</code>
        <span className="spacer" />
        <span className={a.email ? "muted" : "warn"}>{a.email ?? t("never logged in")}</span>
      </div>
      {a.is_default ? (
        <p className="muted small">{t("Source account: the others share its items.")}</p>
      ) : a.wsl ? (
        <p className="muted small">{t("WSL account: read-only for now.")}</p>
      ) : (
        <>
          <div className="row">
            <span className="muted small label">{t("Shared")}</span>
            {a.shared.map((s) => (
              <button
                key={s.item}
                className={`chip-toggle${s.link ? " on" : ""}`}
                disabled={s.own}
                title={s.own ? t("This account already has its own {0}", s.item) : s.link ? t("Link to {0}", s.link) : t("Not shared")}
                onClick={() => run(api.setShared(a.config_dir, s.item, !s.link))}
              >
                {s.item}
                {s.own && t(" · own")}
              </button>
            ))}
          </div>
          {shellAliases && <div className="row">
            <span className="muted small label">Alias</span>
            <code>claude-{a.name}</code>
            {a.alias.kind === "manual" ? (
              <span className="muted small">{t("defined by hand, ~/.zshrc line {0}", a.alias.line)}</span>
            ) : (
              <label className="switch">
                <input type="checkbox" checked={a.alias.kind === "managed"} onChange={(e) => run(api.setAlias(a.config_dir, e.target.checked))} />
                {t("in ~/.zshrc")}
              </label>
            )}
          </div>}
        </>
      )}
    </section>
  );
}

function NewAccount({ run }: { run: (p: Promise<Account[]>, success?: string) => Promise<void> }) {
  const [name, setName] = useState("");
  const [share, setShare] = useState(new Set(SHAREABLE));
  const [alias, setAlias] = useState(shellAliases);
  const valid = /^[a-z0-9_-]+$/.test(name) && name !== "default";

  const create = () =>
    run(
      api.createAccount(name, [...share], alias),
      alias
        ? t("Account created. Open a new terminal, run claude-{0} then /login.", name)
        : t("Account created. Run CLAUDE_CONFIG_DIR=~/.claude-{0} claude then /login.", name),
    ).then(() => setName(""));

  return (
    <section className="account new">
      <strong>{t("New account")}</strong>
      <div className="row">
        <input value={name} onChange={(e) => setName(e.target.value.toLowerCase())} placeholder={t("name, e.g. client-x")} spellCheck={false} />
        <code className="muted">~/.claude-{name || "…"}</code>
      </div>
      <div className="row">
        <span className="muted small label">{t("Share")}</span>
        {SHAREABLE.map((item) => (
          <button
            key={item}
            className={`chip-toggle${share.has(item) ? " on" : ""}`}
            onClick={() => {
              const next = new Set(share);
              next.has(item) ? next.delete(item) : next.add(item);
              setShare(next);
            }}
          >
            {item}
          </button>
        ))}
      </div>
      {shellAliases && (
        <label className="switch">
          <input type="checkbox" checked={alias} onChange={(e) => setAlias(e.target.checked)} />
          {t("Add the alias")} <code>claude-{name || "…"}</code> {t("to ~/.zshrc")}
        </label>
      )}
      <button className="primary" disabled={!valid} onClick={create}>
        {t("Create account")}
      </button>
    </section>
  );
}
