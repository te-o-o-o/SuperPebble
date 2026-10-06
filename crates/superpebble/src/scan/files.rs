//! Markdown-based elements: skills, agents, commands, CLAUDE.md memory.

use super::{file_name, tokens, Scan};
use crate::model::{Kind, Scope, Severity};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Skills, agents and commands living under `dir` (a config dir, `.claude/` or a plugin root).
pub fn dir_items(s: &mut Scan, dir: &Path, scope: Scope, parent: Option<&str>) {
    let skills = dir.join("skills");
    for sub in subdirs(&skills) {
        if skill_dir(s, &sub, scope, parent, 1) == 0 {
            // A top-level skill folder with no SKILL.md anywhere below.
            let id = s.push(Kind::Skill, scope, &sub, &file_name(&sub)).id.clone();
            set_parent(s, parent);
            s.issue(
                "invalid-skill",
                Severity::Error,
                &id,
                &["skill", &file_name(&sub), "SKILL.md missing"],
            );
        }
    }
    for (kind, sub) in [(Kind::Agent, "agents"), (Kind::Command, "commands")] {
        let root = dir.join(sub);
        for path in md_files(&root, 2) {
            let fallback = path.strip_prefix(&root).unwrap().with_extension("");
            let fallback = fallback.to_string_lossy().replace('/', ":");
            md_node(s, kind, scope, &path, &fallback, parent);
        }
    }
}

/// Returns the number of skills found. Descends into bucket folders like `skills/synced/<id>/`.
fn skill_dir(s: &mut Scan, dir: &Path, scope: Scope, parent: Option<&str>, depth: u8) -> usize {
    let md = dir.join("SKILL.md");
    if md.is_file() {
        md_node(s, Kind::Skill, scope, &md, &file_name(dir), parent);
        return 1;
    }
    if depth >= 3 {
        return 0;
    }
    subdirs(dir).iter().map(|d| skill_dir(s, d, scope, parent, depth + 1)).sum()
}

fn md_node(s: &mut Scan, kind: Kind, scope: Scope, path: &Path, fallback: &str, parent: Option<&str>) {
    let text = std::fs::read_to_string(path).unwrap_or_default();
    let (raw, fm) = frontmatter(&text);
    let name = fm.as_ref().ok().and_then(|v| v["name"].as_str()).unwrap_or(fallback).to_string();
    let description = fm.as_ref().ok().and_then(|v| v["description"].as_str()).unwrap_or("").to_string();
    let n = s.push(kind, scope, path, &name);
    n.tokens = Some(tokens(&name) + tokens(&description));
    n.meta = json!({ "frontmatter": raw, "description": description });
    let id = n.id.clone();
    set_parent(s, parent);
    // Commands and agents work without frontmatter; a skill needs it.
    match fm {
        Err(e) if kind == Kind::Skill || raw.is_some() => {
            s.issue(
                "invalid-skill",
                Severity::Error,
                &id,
                &[&format!("{kind:?}").to_lowercase(), &name, &e],
            );
        }
        _ => {}
    }
}

fn set_parent(s: &mut Scan, parent: Option<&str>) {
    s.nodes.last_mut().unwrap().parent = parent.map(String::from);
}

/// Raw frontmatter text and its parsed value.
fn frontmatter(text: &str) -> (Option<String>, Result<Value, String>) {
    let Some(rest) = text
        .strip_prefix("---")
        .and_then(|r| r.strip_prefix('\n').or(r.strip_prefix("\r\n")))
    else {
        return (None, Err("frontmatter missing".into()));
    };
    let Some(end) = rest.find("\n---") else {
        return (None, Err("frontmatter not closed".into()));
    };
    let raw = &rest[..end];
    let parsed = serde_yaml::from_str::<Value>(raw).map_err(|e| e.to_string());
    (Some(format!("---\n{raw}\n---")), parsed)
}

/// CLAUDE.md files, with `@path` imports counted in the token estimate.
pub fn memory(s: &mut Scan) {
    let mut files = vec![(Scope::User, s.ctx.config_dir.join("CLAUDE.md"))];
    if let Some(p) = &s.ctx.project {
        files.push((Scope::Project, p.join("CLAUDE.md")));
        if let Some(d) = s.ctx.project_claude() {
            files.push((Scope::Project, d.join("CLAUDE.md")));
        }
        files.push((Scope::Local, p.join("CLAUDE.local.md")));
    }
    for (scope, path) in files {
        if !path.is_file() {
            continue;
        }
        let mut seen = HashSet::new();
        let total = with_imports(&path, &mut seen, 0, &s.ctx.home());
        let imports: Vec<_> = seen.into_iter().filter(|p| *p != path).collect();
        let n = s.push(Kind::Config, scope, &path, &file_name(&path));
        n.tokens = Some(tokens(&total));
        n.meta = json!({ "imports": imports });
    }
}

fn with_imports(path: &Path, seen: &mut HashSet<PathBuf>, depth: u8, home: &Path) -> String {
    if depth > 5 || !seen.insert(path.to_path_buf()) {
        return String::new();
    }
    let text = std::fs::read_to_string(path).unwrap_or_default();
    let base = path.parent().unwrap_or(Path::new("/"));
    let mut out = text.clone();
    for word in text.split_whitespace() {
        let Some(target) = word.strip_prefix('@') else { continue };
        let target = match target.strip_prefix("~/") {
            Some(rest) => home.join(rest),
            None => base.join(target),
        };
        if target.is_file() {
            out += &with_imports(&target, seen, depth + 1, home);
        }
    }
    out
}

fn subdirs(dir: &Path) -> Vec<PathBuf> {
    let mut v: Vec<_> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir() && !file_name(p).starts_with('.'))
        .collect();
    v.sort();
    v
}

fn md_files(dir: &Path, depth: u8) -> Vec<PathBuf> {
    let mut v: Vec<_> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file() && p.extension().is_some_and(|e| e == "md"))
        .collect();
    v.sort();
    if depth > 1 {
        for d in subdirs(dir) {
            v.extend(md_files(&d, depth - 1));
        }
    }
    v
}

#[cfg(test)]
mod tests {
    use super::frontmatter;

    #[test]
    fn parses_folded_description() {
        let (raw, v) = frontmatter("---\nname: x\ndescription: >\n  a\n  b\n---\nbody");
        assert!(raw.is_some());
        assert_eq!(v.unwrap()["description"], "a b");
    }

    #[test]
    fn missing_frontmatter_is_error() {
        assert!(frontmatter("# title").1.is_err());
        assert!(frontmatter("---\nname: [\n---\n").1.is_err());
    }
}
