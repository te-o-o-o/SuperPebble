use std::path::PathBuf;
use superpebble::{scan, ScanContext};

const USAGE: &str = "usage: superpebble <scan|doctor> [--config DIR] [--project DIR | --no-project]

  scan     print the effective config graph as JSON
  doctor   list problems, exit 1 if any error

  --config   account config dir (default: $CLAUDE_CONFIG_DIR or ~/.claude)
  --project  project dir (default: current dir)";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let flag = |name: &str| args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).map(PathBuf::from);
    let ctx = ScanContext {
        config_dir: flag("--config").unwrap_or_else(ScanContext::default_config_dir),
        project: if args.iter().any(|a| a == "--no-project") {
            None
        } else {
            flag("--project").or_else(|| std::env::current_dir().ok())
        },
    };

    match args.first().map(String::as_str) {
        Some("scan") => println!("{}", serde_json::to_string_pretty(&scan(&ctx)).unwrap()),
        Some("doctor") => {
            let g = scan(&ctx);
            for i in &g.issues {
                println!("{:<8} {}", format!("{:?}", i.severity).to_lowercase(), i.message);
            }
            println!("{} nodes, {} problems", g.nodes.len(), g.issues.len());
            if g.issues.iter().any(|i| i.severity == superpebble::model::Severity::Error) {
                std::process::exit(1);
            }
        }
        _ => {
            eprintln!("{USAGE}");
            std::process::exit(2);
        }
    }
}
