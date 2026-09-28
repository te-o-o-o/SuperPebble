import { useEffect, useState } from "react";
import { COLORS, relTime } from "../theme";
import type { Account } from "../types";
import { home } from "./home";
import { WindowControls, macTitleBar } from "./WindowControls";

export function TopBar(props: {
  accounts: Account[];
  account: string;
  onAccount: (dir: string) => void;
  projects: string[];
  project: string | null;
  onProject: (p: string | null) => void;
  scannedAt?: number;
  onRescan: () => void;
}) {
  // Re-render every 30s so "scan il y a X" stays true.
  const [, tick] = useState(0);
  useEffect(() => {
    const t = setInterval(() => tick((n) => n + 1), 30000);
    return () => clearInterval(t);
  }, []);

  return (
    <header className={`topbar${macTitleBar ? " mac" : ""}`} data-tauri-drag-region>
      <div className="brand" data-tauri-drag-region>
        <svg width="20" height="16" viewBox="0 0 24 20">
          <ellipse cx="12" cy="10" rx="10.5" ry="8" fill="none" stroke={COLORS.center} strokeWidth="2.2" />
        </svg>
        SuperPebble
      </div>
      <span className="sep">/</span>
      <label className="select" title="Compte">
        <select value={props.account} onChange={(e) => props.onAccount(e.target.value)}>
          {props.accounts.map((a) => (
            <option key={a.config_dir} value={a.config_dir}>
              {a.name}
            </option>
          ))}
        </select>
      </label>
      <span className="sep">/</span>
      <label className="select" title="Projet">
        <select className="mono" value={props.project ?? ""} onChange={(e) => props.onProject(e.target.value || null)}>
          <option value="">aucun projet</option>
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
          scan {relTime(props.scannedAt)}
        </span>
      )}
      <button className="ghost" onClick={props.onRescan} title="Rescanner">
        <svg viewBox="0 0 16 16"><path d="M13.5 8a5.5 5.5 0 1 1-1.6-3.9M13.5 2.5v3h-3" /></svg>
      </button>
      <button className="ghost" disabled title="Snapshot : arrive avec le ménage (M2)">
        Snapshot
      </button>
      <WindowControls />
    </header>
  );
}
