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
/** A choice made with the switch wins; otherwise the system language. */
export const lang: Lang = (stored ?? navigator.language).startsWith("fr") ? "fr" : "en";
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
  "Skills & commands": "Skills & commandes",
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
  "Context budget": "Budget de contexte",
  "loaded at startup, estimated": "chargés au démarrage, estimation",
  "{0} servers, not counted": "{0} serveurs, non comptés",
  "Heaviest": "Les plus lourds",
  "Estimate: characters ÷ 4 of CLAUDE.md files with their @imports, and of the name and description of each skill, command and agent. MCP tool definitions are not counted: reading them would mean starting the servers.":
    "Estimation : caractères ÷ 4 des fichiers CLAUDE.md avec leurs @imports, et du nom et de la description de chaque skill, commande et agent. Les définitions d'outils MCP ne sont pas comptées : il faudrait démarrer les serveurs.",
  "Select a pebble to see its details.": "Sélectionne un galet pour voir son détail.",
  "{0} · scope {1}": "{0} · scope {1}",
  "unknown": "inconnu",
  "Modified": "Modifié",
  "Also in scope {0}:": "Aussi dans le scope {0} :",
  "Server": "Serveur",
  "Enabled": "Actif",
  "Open in editor": "Ouvrir dans l'éditeur",
  "Collapse": "Réduire",
  "Show all": "Voir tout",
  "Zoom in": "Zoomer",
  "Zoom out": "Dézoomer",
  // cleanup & snapshots
  "remove from {0}": "retirer de {0}",
  "Nothing the cleanup can fix.": "Rien que le ménage sache corriger.",
  "{0} other issue(s) need a fix by hand: click them in the bottom bar.": "{0} autre(s) problème(s) à corriger à la main : clique dessus dans la barre du bas.",
  "A snapshot is taken first: undo it from Snapshot. Close your Claude Code sessions first: they rewrite these files too.":
    "Un snapshot est pris avant : annule depuis Snapshot. Ferme d'abord tes sessions Claude Code : elles réécrivent aussi ces fichiers.",
  "Remove {0}": "Retirer {0}",
  "Every change SuperPebble makes is copied first to {0}. Restoring puts those files back, after a snapshot of their current state.":
    "Chaque modification de SuperPebble est d'abord copiée dans {0}. Restaurer remet ces fichiers en place, après un snapshot de leur état actuel.",
  "Take a snapshot now": "Prendre un snapshot",
  "Restore": "Restaurer",
  "Confirm restore": "Confirmer",
  "Unknown snapshot.": "Snapshot inconnu.",
  "Pick a project: Claude Code turns MCP servers off per project.": "Choisis un projet : Claude Code désactive les serveurs MCP projet par projet.",
  "Plugin content and managed settings cannot be turned off.": "Le contenu des plugins et les réglages managed ne se désactivent pas.",
  "Claude Code has no off switch for this element.": "Claude Code n'a pas d'interrupteur pour cet élément.",
  "Another settings file overrides this: open it to change it there.": "Un autre fichier de réglages l'emporte : ouvre-le pour changer ça là-bas.",
  // transfer
  "Move…": "Déplacer…",
  "Move or copy {0}": "Déplacer ou copier {0}",
  "Mode": "Mode",
  "Move": "Déplacer",
  "Copy": "Copier",
  "Close your Claude Code sessions first: they rewrite these files too.": "Ferme d'abord tes sessions Claude Code : elles réécrivent aussi ces fichiers.",
  "It holds a plaintext secret and .mcp.json is often committed. Move it anyway.": "Il contient un secret en clair et .mcp.json est souvent commité. Le déplacer quand même.",
  "WSL accounts are read-only.": "Les comptes WSL sont en lecture seule.",
  "Plugin content and managed settings cannot be moved.": "Le contenu des plugins et les réglages managed ne se déplacent pas.",
  "This element cannot be moved yet.": "Cet élément ne se déplace pas encore.",
  "This element has no such scope.": "Cet élément n'existe pas dans ce scope.",
  "Element not found: rescan and retry.": "Élément introuvable : rescanne et réessaie.",
  "Unexpected JSON shape, left untouched.": "Structure JSON inattendue, rien n'a été modifié.",
  "Already there.": "Il y est déjà.",
  "Claude Code installs it on that account itself: this needs the network.": "Claude Code l'installe lui-même sur ce compte : il faut le réseau.",
  "A plugin changes account only from and to the user scope.": "Un plugin ne change de compte que du scope user vers le scope user.",
  "Unknown marketplace source.": "Source de marketplace inconnue.",
  "This hook already exists there.": "Ce hook existe déjà à cet endroit.",
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
