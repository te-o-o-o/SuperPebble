//! Accounts = config dirs: `~/.claude` (default) and `~/.claude-<name>`.
//! Sharing = a symlink from the account to the same item in the default account.
//! MCP servers, plugins and credentials always stay per account.

mod zshrc;

use crate::scan::{home, ScanContext};
use crate::snapshot;
use serde::Serialize;
use std::path::{Path, PathBuf};

/// What an account can share with the default one.
pub const SHAREABLE: [&str; 4] = ["skills", "agents", "commands", "CLAUDE.md"];

#[derive(Serialize)]
pub struct Account {
    pub name: String,
    pub config_dir: PathBuf,
    pub is_default: bool,
    /// From the OAuth profile Claude Code caches in `.claude.json`; None = never logged in.
    pub email: Option<String>,
    pub shared: Vec<Shared>,
    pub alias: Alias,
}

#[derive(Serialize)]
pub struct Shared {
    pub item: &'static str,
    /// Symlink target when shared, None when the account has its own (or nothing).
    pub link: Option<PathBuf>,
    /// The account has its own real content here, so sharing would need it moved first.
    pub own: bool,
}

#[derive(Serialize, PartialEq, Debug)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Alias {
    None,
    Managed,
    Manual { line: usize },
}

fn default_dir() -> PathBuf {
    home().join(".claude")
}

fn zshrc_path() -> PathBuf {
    home().join(".zshrc")
}

pub fn list() -> Vec<Account> {
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(home())
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir() && (*p == default_dir() || is_named_account(p)))
        .collect();
    let current = ScanContext::default_config_dir();
    if !dirs.contains(&current) {
        dirs.push(current);
    }
    dirs.sort();

    let aliases = zshrc::parse(&std::fs::read_to_string(zshrc_path()).unwrap_or_default());
    dirs.into_iter()
        .map(|d| {
            let is_default = d == default_dir();
            let n = d.file_name().unwrap_or_default().to_string_lossy();
            let name = if is_default { "default".to_string() } else { n.strip_prefix(".claude-").unwrap_or(&n).to_string() };
            let alias = match (aliases.manual.get(&name), aliases.managed.contains_key(&name)) {
                (Some(&line), _) => Alias::Manual { line },
                (None, true) => Alias::Managed,
                _ => Alias::None,
            };
            let shared = SHAREABLE
                .iter()
                .map(|item| {
                    let link = std::fs::read_link(d.join(item)).ok();
                    Shared { item, own: link.is_none() && d.join(item).exists(), link }
                })
                .collect();
            Account { email: email(&d), shared, alias, is_default, name, config_dir: d }
        })
        .collect()
}

fn email(config_dir: &Path) -> Option<String> {
    let ctx = ScanContext { config_dir: config_dir.to_path_buf(), project: None };
    let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(ctx.claude_json()).ok()?).ok()?;
    v["oauthAccount"]["emailAddress"].as_str().map(String::from)
}

/// Writes are only ever allowed in `~/.claude-<name>`, never in the default account.
fn is_named_account(p: &Path) -> bool {
    p.parent() == Some(&home()) && p.file_name().is_some_and(|n| n.to_string_lossy().starts_with(".claude-"))
}

fn valid_name(name: &str) -> bool {
    !name.is_empty() && name != "default" && name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
}

pub fn create(name: &str, share: &[String], alias: bool) -> Result<(), String> {
    if !valid_name(name) {
        return Err("Nom invalide : minuscules, chiffres, - et _ uniquement.".into());
    }
    let dir = home().join(format!(".claude-{name}"));
    if dir.exists() {
        return Err(format!("{} existe déjà.", dir.display()));
    }
    if alias && cfg!(windows) {
        return Err("Les alias shell (~/.zshrc) ne sont pas disponibles sous Windows.".into());
    }
    if let Some(line) = manual_alias(name).filter(|_| alias) {
        return Err(format!("claude-{name} est déjà défini à la main dans ~/.zshrc (ligne {line})."));
    }
    std::fs::create_dir(&dir).map_err(|e| e.to_string())?;
    for item in share {
        set_shared(&dir, item, true)?;
    }
    if alias {
        set_alias(&dir, true)?;
    }
    Ok(())
}

/// Shares (`on`) or un-shares an item. Never deletes real content: an account that already has
/// its own `skills/` must move it away first.
pub fn set_shared(config_dir: &Path, item: &str, on: bool) -> Result<(), String> {
    if !is_named_account(config_dir) || !SHAREABLE.contains(&item) {
        return Err("Partage impossible pour ce compte ou cet élément.".into());
    }
    let link = config_dir.join(item);
    let is_link = std::fs::symlink_metadata(&link).is_ok_and(|m| m.file_type().is_symlink());
    if !is_link && link.exists() {
        return Err(format!("{item} existe déjà dans ce compte : déplace-le ou vide-le d'abord."));
    }
    if is_link {
        std::fs::remove_file(&link).map_err(|e| e.to_string())?;
    }
    if !on {
        return Ok(());
    }
    let src = default_dir().join(item);
    if !src.exists() {
        let made = if item == "CLAUDE.md" { std::fs::write(&src, "") } else { std::fs::create_dir_all(&src) };
        made.map_err(|e| e.to_string())?;
    }
    symlink(&src, &link).map_err(|e| e.to_string())
}

#[cfg(unix)]
fn symlink(src: &Path, link: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(src, link)
}

/// Needs Developer Mode (or admin) on Windows, like any symlink there.
#[cfg(windows)]
fn symlink(src: &Path, link: &Path) -> std::io::Result<()> {
    if src.is_dir() {
        std::os::windows::fs::symlink_dir(src, link)
    } else {
        std::os::windows::fs::symlink_file(src, link)
    }
}

fn manual_alias(name: &str) -> Option<usize> {
    zshrc::parse(&std::fs::read_to_string(zshrc_path()).unwrap_or_default()).manual.get(name).copied()
}

/// Adds or removes `claude-<name>` in our `~/.zshrc` block, after a snapshot of the file.
pub fn set_alias(config_dir: &Path, on: bool) -> Result<(), String> {
    if !is_named_account(config_dir) {
        return Err("Alias impossible pour ce compte.".into());
    }
    if cfg!(windows) {
        return Err("Les alias shell (~/.zshrc) ne sont pas disponibles sous Windows.".into());
    }
    let name = config_dir.file_name().unwrap_or_default().to_string_lossy().trim_start_matches(".claude-").to_string();
    let path = zshrc_path();
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    let aliases = zshrc::parse(&text);
    if let Some(line) = aliases.manual.get(&name).filter(|_| on) {
        return Err(format!("claude-{name} est déjà défini à la main dans ~/.zshrc (ligne {line})."));
    }
    let mut managed = aliases.managed;
    if on {
        managed.insert(name.clone(), format!("$HOME/.claude-{name}"));
    } else {
        managed.remove(&name);
    }
    let new = zshrc::render(&text, &managed);
    if new == text {
        return Ok(());
    }
    snapshot::save(&[&path], &format!("alias claude-{name}")).map_err(|e| e.to_string())?;
    snapshot::write_atomic(&path, &new).map_err(|e| e.to_string())
}

/// Projects Claude Code has opened with this account, still present on disk.
pub fn projects(config_dir: PathBuf) -> Vec<PathBuf> {
    let ctx = ScanContext { config_dir, project: None };
    let text = std::fs::read_to_string(ctx.claude_json()).unwrap_or_default();
    let v: serde_json::Value = serde_json::from_str(&text).unwrap_or_default();
    let mut out: Vec<PathBuf> = v["projects"].as_object().into_iter().flatten().map(|(k, _)| PathBuf::from(k)).filter(|p| p.is_dir()).collect();
    out.sort();
    out
}
