import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { t } from "./i18n";
import type { Account, Graph } from "./types";

// Outside Tauri (plain `npm run dev`), serve the graph from `npm run fixture` so the UI can be worked on in a browser.
const inTauri = "__TAURI_INTERNALS__" in window;

const tauriOnly = <T>(cmd: string, args: Record<string, unknown>): Promise<T> =>
  inTauri ? invoke(cmd, args) : Promise.reject(t("Available in the app only."));

export const api = {
  accounts: (): Promise<Account[]> =>
    inTauri
      ? invoke("accounts")
      : Promise.resolve([{ name: "demo", config_dir: "~/.claude", is_default: true, email: null, shared: [], alias: { kind: "none" } }]),
  createAccount: (name: string, share: string[], alias: boolean): Promise<Account[]> => tauriOnly("create_account", { name, share, alias }),
  setShared: (configDir: string, item: string, on: boolean): Promise<Account[]> => tauriOnly("set_shared", { configDir, item, on }),
  setAlias: (configDir: string, on: boolean): Promise<Account[]> => tauriOnly("set_alias", { configDir, on }),
  projects: (configDir: string): Promise<string[]> => (inTauri ? invoke("projects", { configDir }) : Promise.resolve([])),
  scan: (configDir: string, project: string | null): Promise<Graph> =>
    inTauri ? invoke("scan", { configDir, project }) : fetch("/graph.json").then((r) => r.json()),
  /** Watches the files of this account/project; `onChange` fires on every edit. */
  watch: async (configDir: string, project: string | null, onChange: () => void) => {
    if (!inTauri) return () => {};
    await invoke("watch", { configDir, project });
    return listen("config-changed", onChange);
  },
  openPath: (path: string): Promise<void> => (inTauri ? invoke("open_path", { path }) : Promise.resolve()),
};
