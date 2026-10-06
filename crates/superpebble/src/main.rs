use serde_json::json;
use std::path::PathBuf;
use superpebble::model::{Graph, Severity};
use superpebble::{scan, ScanContext};

const USAGE: &str = "usage: superpebble <scan|doctor|weight|accounts> [options]

  scan      print the effective config graph as JSON
  doctor    list problems
  weight    estimate the context Claude Code loads at startup
  accounts  list accounts (config dirs), their sharing and shell alias

  --config DIR             account config dir (default: $CLAUDE_CONFIG_DIR or ~/.claude)
  --project DIR            project dir (default: current dir)
  --no-project             scan the account alone
  --json                   doctor, weight: machine-readable output
  --severity warn|error    doctor: ignore problems below this level
  --quiet                  doctor: print the problems only, no summary

doctor exit codes: 0 no problem, 1 warnings only, 2 errors, 64 bad usage.";

const METHOD: &str = "chars/4 of what Claude Code loads at startup: CLAUDE.md files with their @imports in full, \
and the name + description of each skill, command and agent. MCP tool definitions are not counted: \
reading them would mean starting the servers.";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let has = |name: &str| args.iter().any(|a| a == name);
    let flag = |name: &str| args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned();
    let ctx = ScanContext {
        config_dir: flag("--config").map_or_else(ScanContext::default_config_dir, PathBuf::from),
        project: if has("--no-project") {
            None
        } else {
            flag("--project").map(PathBuf::from).or_else(|| std::env::current_dir().ok())
        },
    };

    match args.first().map(String::as_str) {
        Some("accounts") => print(json!(superpebble::accounts::list())),
        Some("scan") => print(json!(scan(&ctx))),
        Some("doctor") => {
            let min = match flag("--severity").as_deref() {
                None => Severity::Info,
                Some("warn" | "warning") => Severity::Warning,
                Some("error") => Severity::Error,
                Some(_) => usage(),
            };
            std::process::exit(doctor(&scan(&ctx), min, has("--json"), has("--quiet")));
        }
        Some("weight") => weight(&scan(&ctx), has("--json")),
        _ => usage(),
    }
}

fn print(v: serde_json::Value) {
    println!("{}", serde_json::to_string_pretty(&v).unwrap_or_default());
}

fn usage() -> ! {
    eprintln!("{USAGE}");
    std::process::exit(64);
}

fn doctor(g: &Graph, min: Severity, as_json: bool, quiet: bool) -> i32 {
    let issues: Vec<_> = g.issues.iter().filter(|i| i.severity >= min).collect();
    let count = |s| issues.iter().filter(|i| i.severity == s).count();
    let (errors, warnings) = (count(Severity::Error), count(Severity::Warning));
    if as_json {
        print(json!({ "errors": errors, "warnings": warnings, "issues": issues }));
    } else {
        for i in &issues {
            println!("{:<8} {}", format!("{:?}", i.severity).to_lowercase(), i.message);
        }
        if !quiet {
            println!("{} nodes, {errors} errors, {warnings} warnings", g.nodes.len());
        }
    }
    match (errors, warnings) {
        (0, 0) => 0,
        (0, _) => 1,
        _ => 2,
    }
}

fn weight(g: &Graph, as_json: bool) {
    let b = &g.budget;
    let top: Vec<_> = b.top.iter().filter_map(|id| g.nodes.iter().find(|n| &n.id == id)).collect();
    if as_json {
        print(json!({
            "estimate": true,
            "method": METHOD,
            "total": b.total,
            "categories": { "claude_md": b.claude_md, "skills": b.skills, "agents": b.agents },
            "mcp_servers_not_counted": b.mcp_servers,
            "top": top.iter().map(|n| json!({ "name": n.name, "kind": n.kind, "scope": n.scope, "source": n.source, "tokens": n.tokens })).collect::<Vec<_>>(),
        }));
        return;
    }
    println!("~{} tokens loaded at startup (estimate)\n", b.total);
    for (name, t) in [("CLAUDE.md", b.claude_md), ("skills & commands", b.skills), ("agents", b.agents)] {
        println!("  {name:<18} ~{t}");
    }
    println!("  {:<18} {} servers, not counted\n\nHeaviest:", "MCP", b.mcp_servers);
    for n in top {
        println!(
            "  ~{:<7} {:<8} {}",
            n.tokens.unwrap_or(0),
            format!("{:?}", n.kind).to_lowercase(),
            n.source.display()
        );
    }
    println!("\nEstimate: {METHOD}");
}
