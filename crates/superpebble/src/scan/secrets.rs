//! Finds secret values in MCP and hook configs and masks them in place.
//! Only the *names* of what held a secret are returned, never a value or part of one.

use serde_json::Value;

pub const MASK: &str = "***";

/// `GITHUB_TOKEN`, `--api-key`, `X-API-Key`, `Authorization`: the last word names a secret.
/// Last word only, so `TOKEN_LIMIT`, `API_KEY_FILE` or `AWS_ACCESS_KEY_ID` don't count.
pub fn secret_name(name: &str) -> bool {
    let n = name.trim_start_matches('-').to_ascii_uppercase().replace('-', "_");
    let words: Vec<&str> = n.split('_').filter(|w| !w.is_empty()).collect();
    !words.contains(&"PUBLIC")
        && words.last().is_some_and(|w| {
            ["KEY", "PASSWD", "AUTHORIZATION", "CREDENTIALS"].contains(w)
                || ["TOKEN", "SECRET", "PASSWORD", "APIKEY"].iter().any(|s| w.ends_with(s))
        })
}

/// A value written in the file: not a `${VAR}` resolved at launch, nor a path to a key file.
fn literal(v: &str) -> bool {
    !v.is_empty() && !v.contains("${") && !v.starts_with(['/', '~', '.'])
}

/// Well-known token formats, flagged whatever their key: `sk-ant-…`, `ghp_…`, `AKIA…`.
pub fn token_like(v: &str) -> bool {
    const PREFIXES: [&str; 20] = [
        "sk-",
        "ghp_",
        "gho_",
        "ghu_",
        "ghs_",
        "github_pat_",
        "glpat-",
        "xoxb-",
        "xoxp-",
        "xoxa-",
        "xapp-",
        "AKIA",
        "AIza",
        "npm_",
        "hf_",
        "sk_live_",
        "rk_live_",
        "dop_v1_",
        "lin_api_",
        "gsk_",
    ];
    v.len() >= 20 && !v.contains(char::is_whitespace) && PREFIXES.iter().any(|p| v.starts_with(p))
}

/// An env or header entry whose value is a secret written in clear.
pub fn plaintext(key: &str, value: &Value) -> bool {
    value.as_str().is_some_and(|v| (secret_name(key) && literal(v)) || token_like(v))
}

/// Masks `--api-key VALUE`, `--token=VALUE`, `KEY=VALUE`, `Authorization: …`, URLs and bare tokens.
pub fn mask_args(args: &mut [Value], found: &mut Vec<String>) {
    let mut flag: Option<String> = None;
    for a in args.iter_mut() {
        let Some(s) = a.as_str() else {
            flag = None;
            continue;
        };
        let masked = match flag.take() {
            Some(f) if literal(s) && !s.starts_with('-') => {
                found.push(f);
                MASK.to_string()
            }
            _ => mask_word(s, found),
        };
        if s.starts_with('-') && !s.contains('=') && secret_name(s) {
            flag = Some(s.trim_start_matches('-').to_string());
        }
        *a = masked.into();
    }
}

/// A hook's shell command, word by word. Quoted strings with spaces are not reassembled.
pub fn mask_command(cmd: &str, found: &mut Vec<String>) -> String {
    let mut words: Vec<Value> = cmd.split(' ').map(Value::from).collect();
    mask_args(&mut words, found);
    words.iter().filter_map(Value::as_str).collect::<Vec<_>>().join(" ")
}

fn mask_word(word: &str, found: &mut Vec<String>) -> String {
    let w = word.trim_matches(['\'', '"']);
    if w.contains("://") {
        return mask_url(w, found);
    }
    for sep in ['=', ':'] {
        if let Some((k, v)) = w.split_once(sep) {
            if secret_name(k) && literal(v.trim_start()) {
                found.push(k.trim_start_matches('-').to_string());
                return format!("{k}{sep}{MASK}");
            }
        }
    }
    if token_like(w) {
        found.push("token".into());
        return MASK.into();
    }
    word.to_string()
}

/// `user:password@host`, tokens as path segments and secret query parameters.
pub fn mask_url(url: &str, found: &mut Vec<String>) -> String {
    let (head, query) = url.split_once('?').map_or((url, None), |(h, q)| (h, Some(q)));
    let mut out = head
        .split('/')
        .map(|seg| {
            if let Some((user, host)) = seg.rsplit_once('@').and_then(|(cred, host)| Some((cred.split_once(':')?.0, host))) {
                found.push("url password".into());
                return format!("{user}:{MASK}@{host}");
            }
            if token_like(seg) {
                found.push("url".into());
                return MASK.into();
            }
            seg.to_string()
        })
        .collect::<Vec<_>>()
        .join("/");
    if let Some(q) = query {
        let q: Vec<String> = q
            .split('&')
            .map(|p| match p.split_once('=') {
                Some((k, v)) if (secret_name(k) && literal(v)) || token_like(v) => {
                    found.push(k.to_string());
                    format!("{k}={MASK}")
                }
                _ => p.to_string(),
            })
            .collect();
        out = format!("{out}?{}", q.join("&"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn names() {
        for yes in [
            "GITHUB_TOKEN",
            "--api-key",
            "X-API-Key",
            "Authorization",
            "DB_PASSWORD",
            "AWS_SECRET_ACCESS_KEY",
            "apiKey",
        ] {
            assert!(secret_name(yes), "{yes}");
        }
        for no in [
            "TOKEN_LIMIT",
            "MAX_TOKENS",
            "API_KEY_FILE",
            "AWS_ACCESS_KEY_ID",
            "STRIPE_PUBLIC_KEY",
            "LOG_LEVEL",
            "--keyboard",
        ] {
            assert!(!secret_name(no), "{no}");
        }
    }

    #[test]
    fn env_values() {
        assert!(plaintext("GITHUB_TOKEN", &json!("ghp_x")));
        assert!(plaintext("Authorization", &json!("Bearer x")));
        assert!(plaintext("WHATEVER", &json!("sk-ant-api03-abcdefghijklmnop")));
        assert!(!plaintext("GITHUB_TOKEN", &json!("${GITHUB_TOKEN}")));
        assert!(!plaintext("SSH_KEY", &json!("~/.ssh/id_ed25519")));
        assert!(!plaintext("LOG_LEVEL", &json!("debug")));
    }

    #[test]
    fn masks_without_leaking() {
        let mut found = vec![];
        let mut args = vec![
            json!("-y"),
            json!("@x/server"),
            json!("--api-key"),
            json!("s3cret-a"),
            json!("--token=s3cret-b"),
            json!("API_KEY=s3cret-c"),
            json!("Authorization: Bearer s3cret-d"),
            json!("ghp_s3cretttttttttttttttttt"),
            json!("--token"),
            json!("${TOKEN}"),
        ];
        mask_args(&mut args, &mut found);
        let url = mask_url(
            "https://u:s3cret-e@h.io/sk-s3crettttttttttttttttt/sse?api_key=s3cret-f&mode=x",
            &mut found,
        );
        let cmd = mask_command("curl -H X-Api-Key:s3cret-g https://h.io", &mut found);
        let all = format!("{args:?}{url}{cmd}{found:?}");
        assert!(!all.contains("s3cret"), "{all}");
        assert_eq!(url, "https://u:***@h.io/***/sse?api_key=***&mode=x");
        assert_eq!(cmd, "curl -H X-Api-Key:*** https://h.io");
        assert_eq!(args[1], "@x/server");
        assert_eq!(args[9], "${TOKEN}");
        assert_eq!(found.len(), 9);
    }
}
