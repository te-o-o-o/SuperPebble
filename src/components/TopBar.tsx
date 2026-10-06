import { useEffect, useState } from "react";
import rough from "roughjs";
import { LANGS, lang, setLang, t, type Lang } from "../i18n";
import { COLORS, relTime } from "../theme";
import type { Account } from "../types";
import { home } from "./home";
import { WindowControls, macTitleBar } from "./WindowControls";

/** Same two pencil passes as app-icon.svg, drawn at icon scale and shrunk, with thicker strokes to stay legible. */
const gen = rough.generator();
const one = { disableMultiStroke: true, stroke: COLORS.center, fill: undefined };
const LOGO = [
  gen.ellipse(64, 66, 88.5, 67.5, { ...one, strokeWidth: 9, roughness: 1.9, bowing: 2.2, seed: 24 }),
  gen.ellipse(65.8, 64.8, 91.5, 65.5, { ...one, strokeWidth: 7.5, roughness: 2.2, bowing: 2.6, seed: 91 }),
].flatMap((d) => gen.toPaths(d));

export function TopBar(props: {
  accounts: Account[];
  account: string;
  onAccount: (dir: string) => void;
  projects: string[];
  project: string | null;
  onProject: (p: string | null) => void;
  scannedAt?: number;
  onRescan: () => void;
  /** Absent for read-only (WSL) accounts. */
  onSnapshots?: () => void;
}) {
  // Re-render every 30s so "scanned X ago" stays true.
  const [, tick] = useState(0);
  useEffect(() => {
    const t = setInterval(() => tick((n) => n + 1), 30000);
    return () => clearInterval(t);
  }, []);

  return (
    <header className={`topbar${macTitleBar ? " mac" : ""}`} data-tauri-drag-region>
      <div className="brand" data-tauri-drag-region>
        <svg width="20" height="16" viewBox="14 26 102 80">
          {LOGO.map((p, i) => <path key={i} d={p.d} stroke={p.stroke} strokeWidth={p.strokeWidth} fill="none" strokeLinecap="round" />)}
        </svg>
        SuperPebble
      </div>
      <span className="sep">/</span>
      <label className="select" title={t("Account")}>
        <select value={props.account} onChange={(e) => props.onAccount(e.target.value)}>
          {props.accounts.map((a) => (
            <option key={a.config_dir} value={a.config_dir}>
              {a.name}
            </option>
          ))}
        </select>
      </label>
      <span className="sep">/</span>
      <label className="select" title={t("Project")}>
        <select className="mono" value={props.project ?? ""} onChange={(e) => props.onProject(e.target.value || null)}>
          <option value="">{t("no project")}</option>
          {props.projects.map((p) => (
            <option key={p} value={p}>
              {home(p)}
            </option>
          ))}
        </select>
      </label>
      <span className="spacer" data-tauri-drag-region />
      {props.scannedAt && (
        <span className="status" data-tauri-drag-region>
          {t("scanned {0}", relTime(props.scannedAt))}
        </span>
      )}
      <button className="ghost" onClick={props.onRescan} title={t("Rescan")}>
        <svg viewBox="0 0 16 16"><path d="M13.5 8a5.5 5.5 0 1 1-1.6-3.9M13.5 2.5v3h-3" /></svg>
      </button>
      <button className="ghost" disabled={!props.onSnapshots} onClick={props.onSnapshots}>
        Snapshot
      </button>
      <label className="select" title={t("Language")}>
        <select value={lang} onChange={(e) => setLang(e.target.value as Lang)}>
          {LANGS.map((l) => (
            <option key={l} value={l}>
              {l.toUpperCase()}
            </option>
          ))}
        </select>
      </label>
      <WindowControls />
    </header>
  );
}
