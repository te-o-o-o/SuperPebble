//! Runs in its own process: points $HOME at a temp dir, where snapshots land.

use serde_json::{json, Value};
use std::fs;
use std::path::Path;
use superpebble::edit::{clean_up, set_enabled};
use superpebble::{scan, snapshot, ScanContext};

fn read(p: &Path) -> Value {
    serde_json::from_str(&fs::read_to_string(p).unwrap()).unwrap()
}

#[test]
fn clean_up_toggle_and_restore() {
    let home = std::env::temp_dir().join(format!("sp-edit-{}", std::process::id()));
    let _ = fs::remove_dir_all(&home);
    let (config, project) = (home.join("config"), home.join("project"));
    fs::create_dir_all(&config).unwrap();
    fs::create_dir_all(&project).unwrap();
    std::env::set_var("HOME", &home);
    std::env::set_var("USERPROFILE", &home);

    let hook = |script: &str| json!({ "type": "command", "command": format!("sh /nope/{script}") });
    let settings = json!({ "hooks": { "Stop": [{ "hooks": [hook("a.sh"), hook("b.sh")] }] } });
    fs::write(config.join("settings.json"), settings.to_string()).unwrap();
    // The test binary itself: a command that exists on every OS.
    let ok = json!({ "command": std::env::current_exe().unwrap() });
    let servers = json!({ "mcpServers": { "gone": { "command": "no-such-binary-xyz" }, "ok": ok } });
    fs::write(config.join(".claude.json"), servers.to_string()).unwrap();
    let ctx = ScanContext {
        config_dir: config.clone(),
        project: Some(project),
    };

    let ids: Vec<String> = scan(&ctx).issues.iter().filter(|i| i.fix).flat_map(|i| i.nodes.clone()).collect();
    assert_eq!(ids.len(), 3, "two broken hooks and one orphan");
    clean_up(&ctx, &ids).unwrap();
    assert_eq!(read(&config.join("settings.json"))["hooks"], json!({}));
    assert_eq!(read(&config.join(".claude.json"))["mcpServers"], json!({ "ok": ok }));

    let id = scan(&ctx).nodes.into_iter().find(|n| n.name == "ok").unwrap().id;
    let enabled = || scan(&ctx).nodes.iter().find(|n| n.id == id).unwrap().enabled;
    set_enabled(&ctx, &id, false).unwrap();
    assert!(!enabled());
    set_enabled(&ctx, &id, true).unwrap();
    assert!(enabled());

    // The oldest snapshot holds the files as they were before the cleanup.
    let all = snapshot::list();
    assert_eq!(all.len(), 3);
    snapshot::restore(&all[2].id).unwrap();
    assert!(read(&config.join(".claude.json"))["mcpServers"]["gone"].is_object());
    assert_eq!(snapshot::list().len(), 4, "a restore snapshots first");
    assert!(snapshot::restore("../etc").is_err());
    fs::remove_dir_all(&home).unwrap();
}
