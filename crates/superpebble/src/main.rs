use serde_json::json;
use std::path::PathBuf;
use superpebble::model::{Graph, Kind, Severity};
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
    let print = |v: serde_json::Value| println!("{}", serde_json::to_string_pretty(&v).unwrap_or_default());

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

fn usage() -> ! {
    eprintln!("{USAGE}");
    std::process::exit(64);
}

fn doctor(g: &Graph, min: Severity, as_json: bool, quiet: bool) -> i32 {
    let issues: Vec<_> = g.issues.iter().filter(|i| i.severity >= min).collect();
    let count = |s| issues.iter().filter(|i| i.severity == s).count();
    let (errors, warnings) = (count(Severity::Error), count(Severity::Warning));
    if as_json {
        let v = json!({ "errors": errors, "warnings": warnings, "issues": issues });
        println!("{}", serde_json::to_string_pretty(&v).unwrap_or_default());
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
    // Plugin nodes carry the sum of their children: counting both would count twice.
    let mut loaded: Vec<_> = g
        .nodes
        .iter()
        .filter(|n| n.enabled && n.kind != Kind::Plugin && n.tokens.is_some())
        .collect();
    loaded.sort_by_key(|n| std::cmp::Reverse(n.tokens));
    let sum = |kinds: &[Kind]| {
        loaded
            .iter()
            .filter(|n| kinds.contains(&n.kind))
            .map(|n| n.tokens.unwrap_or(0))
            .sum::<u32>()
    };
    let categories = [
        ("CLAUDE.md", sum(&[Kind::Config])),
        ("skills & commands", sum(&[Kind::Skill, Kind::Command])),
        ("agents", sum(&[Kind::Agent])),
    ];
    let total: u32 = categories.iter().map(|c| c.1).sum();
    let mcp = g.nodes.iter().filter(|n| n.enabled && n.kind == Kind::Mcp).count();
    let top = &loaded[..loaded.len().min(10)];

    if as_json {
        let v = json!({
            "estimate": true,
            "method": METHOD,
            "total": total,
            "categories": categories.iter().map(|(k, v)| (k.to_string(), json!(v))).collect::<serde_json::Map<_, _>>(),
            "mcp_servers_not_counted": mcp,
            "top": top.iter().map(|n| json!({ "name": n.name, "kind": n.kind, "scope": n.scope, "source": n.source, "tokens": n.tokens })).collect::<Vec<_>>(),
        });
        println!("{}", serde_json::to_string_pretty(&v).unwrap_or_default());
        return;
    }
    println!("~{total} tokens loaded at startup (estimate)\n");
    for (name, t) in categories {
        println!("  {name:<18} ~{t}");
    }
    println!("  {:<18} {mcp} servers, not counted\n\nHeaviest:", "MCP");
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
