//! Runs in its own process: points $HOME at a temp dir so nothing real is touched.
use serde_json::{json, Value};
use std::fs;
use std::path::Path;
use superpebble::model::{Kind, Scope};
use superpebble::transfer::transfer;
use superpebble::{scan, ScanContext};

fn read(p: &Path) -> Value {
    serde_json::from_str(&fs::read_to_string(p).unwrap()).unwrap()
}

/// Id of the only `kind` node named `name` in `scope`.
fn id(ctx: &ScanContext, kind: Kind, name: &str, scope: Scope) -> String {
    let g = scan(ctx);
    let n = g.nodes.iter().find(|n| n.kind == kind && n.name == name && n.scope == scope);
    n.unwrap_or_else(|| panic!("no {kind:?} {name} in {scope:?}")).id.clone()
}

#[test]
fn moves_and_copies_across_scopes_and_accounts() {
    let home = std::env::temp_dir().join(format!("sp-transfer-{}", std::process::id()));
    let _ = fs::remove_dir_all(&home);
    let (work, project) = (home.join(".claude-work"), home.join("proj"));
    fs::create_dir_all(work.join("skills/review")).unwrap();
    fs::create_dir_all(&project).unwrap();
    fs::write(work.join("skills/review/SKILL.md"), "---\nname: review\ndescription: x\n---\n").unwrap();
    let hooks = json!({ "hooks": { "Stop": [{ "matcher": "*", "hooks": [{ "type": "command", "command": "a" }, { "type": "command", "command": "b" }] }] } });
    fs::write(work.join("settings.json"), hooks.to_string()).unwrap();
    let mcp = json!({ "mcpServers": { "gh": { "command": "gh", "env": { "GH_TOKEN": "x" } } } });
    fs::write(work.join(".claude.json"), mcp.to_string()).unwrap();
    std::env::set_var("HOME", &home);
    std::env::set_var("USERPROFILE", &home);

    let ctx = ScanContext {
        config_dir: work.clone(),
        project: Some(project.clone()),
    };

    // Skill: user → project, the folder moves whole.
    transfer(&ctx, &id(&ctx, Kind::Skill, "review", Scope::User), &work, Scope::Project, false).unwrap();
    assert!(project.join(".claude/skills/review/SKILL.md").is_file());
    assert!(!work.join("skills/review").exists());

    // MCP: copied to local (same file), then the copy moved to project; the user one stays.
    transfer(&ctx, &id(&ctx, Kind::Mcp, "gh", Scope::User), &work, Scope::Local, true).unwrap();
    let local = &read(&work.join(".claude.json"))["projects"][project.to_str().unwrap()]["mcpServers"]["gh"];
    assert_eq!(local["env"]["GH_TOKEN"], "x", "values are copied, not just keys");
    transfer(&ctx, &id(&ctx, Kind::Mcp, "gh", Scope::Local), &work, Scope::Project, false).unwrap();
    assert!(read(&project.join(".mcp.json"))["mcpServers"]["gh"].is_object());
    let cj = read(&work.join(".claude.json"));
    assert!(cj["mcpServers"]["gh"].is_object() && cj["projects"][project.to_str().unwrap()]["mcpServers"]["gh"].is_null());

    // Same place, or a name already taken: refused, nothing written.
    let err = transfer(&ctx, &id(&ctx, Kind::Mcp, "gh", Scope::User), &work, Scope::Project, true).unwrap_err();
    assert!(err.contains("already exists"), "{err}");

    // Hooks: both land in one "*" group of settings.local.json; the emptied source keeps no group.
    for command in ["a", "b"] {
        let g = scan(&ctx);
        let h = g
            .nodes
            .iter()
            .find(|n| n.kind == Kind::Hook && n.scope == Scope::User && n.meta["command"] == command)
            .unwrap();
        transfer(&ctx, &h.id, &work, Scope::Local, false).unwrap();
    }
    let groups = &read(&project.join(".claude/settings.local.json"))["hooks"]["Stop"];
    assert_eq!(
        groups,
        &json!([{ "matcher": "*", "hooks": [{ "type": "command", "command": "a" }, { "type": "command", "command": "b" }] }])
    );
    assert_eq!(read(&work.join("settings.json")), json!({ "hooks": {} }));

    // Another account: the default one, created on the fly.
    transfer(
        &ctx,
        &id(&ctx, Kind::Skill, "review", Scope::Project),
        &home.join(".claude"),
        Scope::User,
        true,
    )
    .unwrap();
    assert!(home.join(".claude/skills/review/SKILL.md").is_file());

    // Plugin: user → project moves the install entry and `enabledPlugins`; a copy to local adds an entry.
    let install = json!({ "scope": "user", "installPath": work.join("plugins/cache/p"), "version": "1" });
    let registry = work.join("plugins/installed_plugins.json");
    fs::create_dir_all(registry.parent().unwrap()).unwrap();
    fs::write(&registry, json!({ "version": 2, "plugins": { "p@m": [install] } }).to_string()).unwrap();
    fs::write(work.join("settings.json"), json!({ "enabledPlugins": { "p@m": true } }).to_string()).unwrap();
    transfer(&ctx, &id(&ctx, Kind::Plugin, "p", Scope::User), &work, Scope::Project, false).unwrap();
    transfer(&ctx, &id(&ctx, Kind::Plugin, "p", Scope::Project), &work, Scope::Local, true).unwrap();
    let installs = &read(&registry)["plugins"]["p@m"];
    let scopes: Vec<_> = installs
        .as_array()
        .unwrap()
        .iter()
        .map(|i| (&i["scope"], &i["projectPath"]))
        .collect();
    let p = json!(project.to_str().unwrap());
    assert_eq!(scopes, [(&json!("project"), &p), (&json!("local"), &p)]);
    assert_eq!(read(&work.join("settings.json")), json!({ "enabledPlugins": {} }));
    assert_eq!(read(&project.join(".claude/settings.json"))["enabledPlugins"]["p@m"], true);
    assert_eq!(read(&project.join(".claude/settings.local.json"))["enabledPlugins"]["p@m"], true);
    let err = transfer(
        &ctx,
        &id(&ctx, Kind::Plugin, "p", Scope::Local),
        &home.join(".claude"),
        Scope::User,
        true,
    )
    .unwrap_err();
    assert!(err.contains("only from and to the user scope"), "{err}");

    assert!(
        home.join(".superpebble/snapshots").read_dir().unwrap().count() >= 4,
        "one snapshot per JSON edit"
    );
    fs::remove_dir_all(&home).unwrap();
}
