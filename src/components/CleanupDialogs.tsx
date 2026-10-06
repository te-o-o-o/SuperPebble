import { useEffect, useRef, useState } from "react";
import { api } from "../api";
import { issueText, lang, t } from "../i18n";
import type { Graph, Snapshot } from "../types";
import { home } from "./home";

type Base = { open: boolean; account: string; project: string | null; onClose: () => void };

function Dialog({ open, title, onClose, children }: { open: boolean; title: string; onClose: () => void; children: React.ReactNode }) {
  const ref = useRef<HTMLDialogElement>(null);
  useEffect(() => {
    if (open) ref.current?.showModal();
    else ref.current?.close();
  }, [open]);
  return (
    <dialog ref={ref} className="accounts" onClose={onClose} onClick={(e) => e.target === ref.current && onClose()}>
      <header>
        <h2>{title}</h2>
        <button className="ghost" onClick={onClose} aria-label={t("Close")}>
          ✕
        </button>
      </header>
      {open && children}
    </dialog>
  );
}

/** Removes the orphan MCP servers and broken hooks the user keeps checked. */
export function CleanupDialog({ graph, onDone, ...p }: Base & { graph?: Graph; onDone: () => void }) {
  const issues = graph?.issues ?? [];
  const fixable = issues.filter((i) => i.fix);
  const [off, setOff] = useState(new Set<string>());
  const [error, setError] = useState<string>();
  useEffect(() => {
    setOff(new Set());
    setError(undefined);
  }, [p.open]);

  const ids = fixable.map((i) => i.nodes[0]).filter((id) => !off.has(id));
  const source = (id: string) => home(graph?.nodes.find((n) => n.id === id)?.source ?? "");
  const submit = () =>
    api.cleanUp(p.account, p.project, ids).then(
      () => {
        onDone();
        p.onClose();
      },
      (e) => setError(t(String(e))),
    );

  return (
    <Dialog open={p.open} title={t("Clean up")} onClose={p.onClose}>
      <section className="account">
        {fixable.length ? (
          fixable.map((i) => {
            const id = i.nodes[0];
            return (
              <label key={id} className="switch">
                <input
                  type="checkbox"
                  checked={!off.has(id)}
                  onChange={() =>
                    setOff((s) => {
                      const next = new Set(s);
                      next.has(id) ? next.delete(id) : next.add(id);
                      return next;
                    })
                  }
                />
                <span>
                  {issueText(i)}
                  <br />
                  <span className="muted small">{t("remove from {0}", source(id))}</span>
                </span>
              </label>
            );
          })
        ) : (
          <p className="muted small">{t("Nothing the cleanup can fix.")}</p>
        )}
        {issues.length > fixable.length && (
          <p className="muted small">{t("{0} other issue(s) need a fix by hand: click them in the bottom bar.", issues.length - fixable.length)}</p>
        )}
        <p className="muted small">{t("A snapshot is taken first: undo it from Snapshot. Close your Claude Code sessions first: they rewrite these files too.")}</p>
        <button className="primary" disabled={!ids.length} onClick={submit}>
          {t("Remove {0}", ids.length)}
        </button>
      </section>
      {error && <p className="error">{error}</p>}
    </Dialog>
  );
}

const when = (id: string) => new Date(Number(id)).toLocaleString(lang);

/** Takes a snapshot of the config files, or puts an earlier one back. */
export function SnapshotsDialog({ onDone, ...p }: Base & { onDone: () => void }) {
  const [list, setList] = useState<Snapshot[]>([]);
  const [armed, setArmed] = useState<string>();
  const [error, setError] = useState<string>();
  useEffect(() => {
    if (p.open) api.snapshots().then(setList);
    setArmed(undefined);
    setError(undefined);
  }, [p.open]);

  const run = (call: Promise<Snapshot[]>) =>
    call.then(
      (l) => {
        setList(l);
        setArmed(undefined);
        setError(undefined);
        onDone();
      },
      (e) => setError(t(String(e))),
    );

  return (
    <Dialog open={p.open} title="Snapshots" onClose={p.onClose}>
      <p className="muted intro">
        {t("Every change SuperPebble makes is copied first to {0}. Restoring puts those files back, after a snapshot of their current state.", "~/.superpebble/snapshots")}
      </p>
      <button className="primary" onClick={() => run(api.snapshotNow(p.account, p.project))}>
        {t("Take a snapshot now")}
      </button>
      {error && <p className="error">{error}</p>}
      {list.map((s) => (
        <section key={s.id} className="account">
          <div className="account-head">
            <strong>{when(s.id)}</strong>
            <code className="muted">{s.reason}</code>
            <span className="spacer" />
            <button className={armed === s.id ? "primary" : undefined} onClick={() => (armed === s.id ? run(api.restore(s.id)) : setArmed(s.id))}>
              {armed === s.id ? t("Confirm restore") : t("Restore")}
            </button>
          </div>
          <p className="muted small">{s.files.map(home).join(" · ")}</p>
        </section>
      ))}
    </Dialog>
  );
}
