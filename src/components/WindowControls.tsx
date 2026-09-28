import { getCurrentWindow } from "@tauri-apps/api/window";

// Linux only: the window has no native decorations (see tauri.linux.conf.json).
// macOS keeps its own traffic lights over the transparent title bar.
export const customTitleBar = "__TAURI_INTERNALS__" in window && !navigator.userAgent.includes("Mac");
export const macTitleBar = "__TAURI_INTERNALS__" in window && navigator.userAgent.includes("Mac");

export function WindowControls() {
  if (!customTitleBar) return null;
  const w = getCurrentWindow();
  return (
    <div className="window-controls">
      <button onClick={() => w.minimize()} aria-label="Réduire">
        <svg viewBox="0 0 10 10"><path d="M1 5h8" /></svg>
      </button>
      <button onClick={() => w.toggleMaximize()} aria-label="Agrandir">
        <svg viewBox="0 0 10 10"><rect x="1.5" y="1.5" width="7" height="7" rx="1" /></svg>
      </button>
      <button className="close" onClick={() => w.close()} aria-label="Fermer">
        <svg viewBox="0 0 10 10"><path d="M2 2l6 6M8 2l-6 6" /></svg>
      </button>
    </div>
  );
}
