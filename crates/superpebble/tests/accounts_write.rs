//! Runs in its own process: points $HOME at a temp dir so nothing real is touched.
use std::fs;
use superpebble::accounts::{create, list, set_alias, set_shared, Alias};

#[test]
fn create_share_alias_roundtrip() {
    let home = std::env::temp_dir().join(format!("sp-home-{}", std::process::id()));
    let _ = fs::remove_dir_all(&home);
    fs::create_dir_all(home.join(".claude/skills")).unwrap();
    let rc = "export X=1\nclaude-work() { CLAUDE_CONFIG_DIR=~/.claude-work claude \"$@\"; }\n";
    fs::write(home.join(".zshrc"), rc).unwrap();
    std::env::set_var("HOME", &home);
    std::env::remove_var("CLAUDE_CONFIG_DIR");

    // A hand-written alias blocks creation before anything is written.
    assert!(create("work", &[], true).unwrap_err().contains("ligne 2"));
    assert!(!home.join(".claude-work").exists());

    create("perso", &["skills".into(), "CLAUDE.md".into()], true).unwrap();
    let perso = home.join(".claude-perso");
    assert_eq!(fs::read_link(perso.join("skills")).unwrap(), home.join(".claude/skills"));
    assert!(home.join(".claude/CLAUDE.md").is_file(), "missing source is created");
    let zshrc = fs::read_to_string(home.join(".zshrc")).unwrap();
    assert!(zshrc.starts_with(rc) && zshrc.contains("claude-perso() { CLAUDE_CONFIG_DIR=\"$HOME/.claude-perso\""));
    assert_eq!(fs::read_dir(home.join(".superpebble/snapshots")).unwrap().count(), 1);

    let a = list().into_iter().find(|a| a.name == "perso").unwrap();
    assert_eq!(a.alias, Alias::Managed);

    // Real content is never replaced; un-sharing removes only the link.
    fs::create_dir(perso.join("agents")).unwrap();
    assert!(set_shared(&perso, "agents", true).is_err());
    set_shared(&perso, "skills", false).unwrap();
    assert!(!perso.join("skills").exists() && home.join(".claude/skills").is_dir());
    assert!(
        set_shared(&home.join(".claude"), "skills", true).is_err(),
        "default account is read-only"
    );

    set_alias(&perso, false).unwrap();
    assert_eq!(fs::read_to_string(home.join(".zshrc")).unwrap(), rc);
    fs::remove_dir_all(&home).unwrap();
}
