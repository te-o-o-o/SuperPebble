use std::path::PathBuf;
use superpebble::{scan, ScanContext};

#[test]
fn fixture_graph_and_issues() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixture");
    let g = scan(&ScanContext { config_dir: root.join("config"), project: Some(root.join("project")) });

    let names: Vec<_> = g.nodes.iter().map(|n| format!("{:?}:{}", n.kind, n.name)).collect();
    for expected in ["Skill:commit-msg", "Agent:code-reviewer", "Command:deploy-staging", "Mcp:github", "Mcp:notion", "Mcp:old-postgres", "Hook:Stop"] {
        assert!(names.contains(&expected.to_string()), "missing {expected} in {names:?}");
    }

    let mut rules: Vec<_> = g.issues.iter().map(|i| i.rule).collect();
    rules.sort();
    assert_eq!(rules, ["broken-hook", "duplicate", "invalid-skill", "orphan-mcp", "plaintext-secret"]);

    // Secret values never reach the graph.
    let json = serde_json::to_string(&g).unwrap();
    assert!(!json.contains("ghp_fake_for_tests"));
    assert!(json.contains("GITHUB_TOKEN"));
}
