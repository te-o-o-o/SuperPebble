use std::path::PathBuf;
use superpebble::{scan, ScanContext};

#[test]
fn fixture_graph_and_issues() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixture");
    let g = scan(&ScanContext {
        config_dir: root.join("config"),
        project: Some(root.join("project")),
    });

    let names: Vec<_> = g.nodes.iter().map(|n| format!("{:?}:{}", n.kind, n.name)).collect();
    for expected in [
        "Skill:commit-msg",
        "Agent:code-reviewer",
        "Command:deploy-staging",
        "Mcp:github",
        "Mcp:notion",
        "Mcp:old-postgres",
        "Hook:Stop",
    ] {
        assert!(names.contains(&expected.to_string()), "missing {expected} in {names:?}");
    }

    let mut rules: Vec<_> = g.issues.iter().map(|i| i.rule).collect();
    rules.sort();
    rules.dedup();
    assert_eq!(
        rules,
        ["broken-hook", "duplicate", "invalid-skill", "orphan-mcp", "plaintext-secret"]
    );

    // Secret values never reach the graph.
    let json = serde_json::to_string(&g).unwrap();
    for value in ["ghp_fake_for_tests", "fake_arg_secret", "fake_url_secret"] {
        assert!(!json.contains(value), "{value} leaked");
    }
    let secrets = g.issues.iter().filter(|i| i.rule == "plaintext-secret").count();
    assert_eq!(secrets, 3);
    assert!(json.contains("GITHUB_TOKEN"));

    let b = &g.budget;
    assert_eq!(b.total, b.claude_md + b.skills + b.agents);
    assert!(b.claude_md > 0 && b.skills > 0 && b.agents > 0);
    assert_eq!(b.mcp_servers, 3);
    assert!(b.top.len() <= 10);
}
