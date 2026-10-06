# superpebble

Reads what Claude Code actually loads for an account and a project (plugins,
skills, commands, agents, MCP servers, hooks, `CLAUDE.md`, settings) and lints it.
The scanner and CLI behind the [SuperPebble](https://github.com/te-o-o-o/SuperPebble) app.

Read-only: nothing scanned is ever run (no MCP server, hook or script), there is
no network access, and secret values never reach the output, masked or not.

```sh
cargo install superpebble

superpebble doctor              # problems for the current project
superpebble weight              # what the startup context costs
superpebble scan                # the whole graph as JSON
superpebble accounts            # accounts (config dirs), sharing, shell aliases
```

| Option | |
|---|---|
| `--config DIR` | account config dir (default `$CLAUDE_CONFIG_DIR`, else `~/.claude`) |
| `--project DIR` / `--no-project` | project to scan (default: the current dir) |
| `--json` | `doctor`, `weight`: machine-readable output |
| `--severity warn\|error` | `doctor`: ignore problems below this level |
| `--quiet` | `doctor`: the problems only, no summary line |

## doctor

| Rule | Severity | Flags |
|---|---|---|
| `orphan-mcp` | error | an MCP server whose command is not found |
| `broken-hook` | error | a hook whose binary or absolute script path is missing |
| `invalid-skill` | error | a skill without `SKILL.md`, or with broken frontmatter |
| `invalid-json` | error | a config file that cannot be read or parsed |
| `plaintext-secret` | warning | a secret written in clear in an MCP server or hook (env, headers, args, url, command) |
| `duplicate` | warning | the same skill/command, agent or MCP name loaded from several scopes |

Exit codes, for CI:

| Code | Meaning |
|---|---|
| 0 | no problem |
| 1 | warnings, no error |
| 2 | at least one error |
| 64 | bad usage |

`--severity` filters before the exit code is computed: `doctor --severity error`
exits 0 on warnings.

`doctor --json` prints `{"errors": n, "warnings": n, "issues": [...]}`, each issue
being `{"rule", "severity", "nodes", "args", "message", "fix"}`; `fix` is true when
the app's cleanup can remove the element (an orphan MCP server or a broken hook,
outside plugins and managed settings). `rule` and `severity`
are stable; `message` is English text meant for humans.

## weight

An **estimate**, not a tokenizer count: characters / 4 of what Claude Code puts
in context at startup, meaning `CLAUDE.md` files with their `@imports` in full,
plus the name and description of each skill, command and agent (their bodies
load on use). MCP tool definitions are not counted: reading them would mean
starting the servers. Prints the total, the split by category and the 10 heaviest
files.
