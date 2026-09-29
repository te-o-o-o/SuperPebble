import type { Issue } from "./types";

export type Lang = "en" | "fr";
export const LANGS: Lang[] = ["en", "fr"];

const stored = (() => {
  try {
    return localStorage.getItem("lang");
  } catch {
    return null;
  }
})();
export const lang: Lang = stored === "fr" ? "fr" : "en";
document.documentElement.lang = lang;

/** Labels are read once at module load, so switching reloads the page. */
export function setLang(l: Lang) {
  try {
    localStorage.setItem("lang", l);
  } catch {}
  location.reload();
}

/** English text is the key; `{0}`, `{1}`… are filled from `args`. */
export function t(en: string, ...args: (string | number)[]) {
  const s = (lang === "fr" && FR[en]) || en;
  return s.replace(/\{(\d)\}/g, (_, i) => String(args[+i]));
}

export function issueText(i: Issue) {
  if (lang === "en" || !FR_ISSUE[i.rule]) return i.message;
  const args = i.rule === "invalid-skill" ? [t(KIND[i.args[0]] ?? i.args[0]), i.args[1], t(i.args[2])] : i.args;
  return FR_ISSUE[i.rule].replace(/\{(\d)\}/g, (_, k) => args[+k] ?? "");
}

const KIND: Record<string, string> = { skill: "Skill", agent: "Agent", command: "Command" };

const FR_ISSUE: Record<string, string> = {
  "orphan-mcp": "MCP orphelin : {0} (commande introuvable)",
  "plaintext-secret": "Secret en clair : {0} ({1})",
  "broken-hook": "Hook cassé : {0} ({1} introuvable)",
  duplicate: "Doublon : {0}",
  "invalid-skill": "{0} invalide : {1} ({2})",
  "invalid-json": "JSON illisible : {0} ({1})",
};

const FR: Record<string, string> = {
  // theme
  "Config": "Conf",
  "MCP servers": "Serveurs MCP",
  "Agents & commands": "Agents & commandes",
  "Config files": "Fichiers de conf",
  "project": "projet",
  "MCP server": "Serveur MCP",
  "Command": "Commande",
  "Config file": "Fichier de conf",
  "not found": "introuvable",
  "broken": "cassé",
  "duplicate": "doublon",
  "invalid": "invalide",
  "unreadable": "illisible",
  "just now": "à l'instant",
  // layout
  "‹ collapse": "‹ réduire",
  "+ {0} more": "+ {0} autres",
  // app
  "Available in the app only.": "Disponible uniquement dans l'app.",
  // top bar
  "Account": "Compte",
  "Project": "Projet",
  "no project": "aucun projet",
  "scanned {0}": "scan {0}",
  "Rescan": "Rescanner",
  "Snapshot: comes with the cleanup (M2)": "Snapshot : arrive avec le ménage (M2)",
  "Language": "Langue",
  "Minimize": "Réduire",
  "Maximize": "Agrandir",
  "Close": "Fermer",
  // sidebar
  "soon": "bientôt",
  "Show": "Afficher",
  "broken / orphan": "cassé / orphelin",
  "Manage accounts": "Gérer les comptes",
  // panels
  "Branch": "Branche",
  "Items": "Éléments",
  "At startup": "Au démarrage",
  "Content": "Contenu",
  "Select a pebble to see its details.": "Sélectionne un galet pour voir son détail.",
  "{0} · scope {1}": "{0} · scope {1}",
  "unknown": "inconnu",
  "Modified": "Modifié",
  "Also in scope {0}:": "Aussi dans le scope {0} :",
  "Server": "Serveur",
  "Read-only until the cleanup (M2)": "Lecture seule jusqu'au ménage (M2)",
  "Enabled": "Actif",
  "read-only": "lecture seule",
  "Open in editor": "Ouvrir dans l'éditeur",
  "Comes in M2": "Arrive en M2",
  "Delete…": "Supprimer…",
  "Collapse": "Réduire",
  "Show all": "Voir tout",
  "Zoom in": "Zoomer",
  "Zoom out": "Dézoomer",
  // issue bar
  "1 issue to review": "1 point à revoir",
  "{0} issues to review": "{0} points à revoir",
  "Nothing to report": "Rien à signaler",
  "Clean up": "Faire le ménage",
  // issue reasons
  "SKILL.md missing": "SKILL.md absent",
  "frontmatter missing": "frontmatter absent",
  "frontmatter not closed": "frontmatter non fermé",
  // accounts
  "Accounts": "Comptes",
  "never logged in": "jamais connecté",
  "Source account: the others share its items.": "Compte source : les autres partagent ses éléments.",
  "WSL account: read-only for now.": "Compte WSL : lecture seule pour l'instant.",
  "Shared": "Partagé",
  "This account already has its own {0}": "Ce compte a déjà son propre {0}",
  "Link to {0}": "Lien vers {0}",
  "Not shared": "Non partagé",
  " · own": " · propre",
  "defined by hand, ~/.zshrc line {0}": "défini à la main, ~/.zshrc ligne {0}",
  "in ~/.zshrc": "dans ~/.zshrc",
  "Account created. Open a new terminal, run claude-{0} then /login.":
    "Compte créé. Ouvre un nouveau terminal, lance claude-{0} puis /login.",
  "Account created. Run CLAUDE_CONFIG_DIR=~/.claude-{0} claude then /login.":
    "Compte créé. Lance CLAUDE_CONFIG_DIR=~/.claude-{0} claude puis /login.",
  "New account": "Nouveau compte",
  "name, e.g. client-x": "nom, ex. client-x",
  "Share": "Partager",
  "Add the alias": "Ajouter l'alias",
  "to ~/.zshrc": "dans ~/.zshrc",
  "Create account": "Créer le compte",
  // backend errors
  "Invalid name: lowercase letters, digits, - and _ only.": "Nom invalide : minuscules, chiffres, - et _ uniquement.",
  "This account or item cannot be shared.": "Partage impossible pour ce compte ou cet élément.",
  "This account cannot have an alias.": "Alias impossible pour ce compte.",
  "Shell aliases (~/.zshrc) are not available on Windows.": "Les alias shell (~/.zshrc) ne sont pas disponibles sous Windows.",
  "path unknown to the last scan": "chemin inconnu du dernier scan",
};
