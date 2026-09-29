//! Claude Code installed inside WSL, seen from Windows through `\\wsl.localhost\<distro>`.
//! Config files there hold Linux paths (`/home/theo/proj`): they are translated both ways.

use std::path::{Path, PathBuf};

/// `\\wsl.localhost\Ubuntu` when `p` lives in a WSL distro.
pub fn root(p: &Path) -> Option<String> {
    let s = p.to_str()?;
    let rest = s.strip_prefix(r"\\wsl.localhost\").or_else(|| s.strip_prefix(r"\\wsl$\"))?;
    let distro = rest.split('\\').next().filter(|d| !d.is_empty())?;
    Some(s[..s.len() - rest.len() + distro.len()].to_string())
}

/// `Ubuntu` for `\\wsl.localhost\Ubuntu\home\theo\.claude`.
pub fn distro(p: &Path) -> Option<String> {
    root(p).and_then(|r| r.rsplit('\\').next().map(String::from))
}

/// `/home/theo` → `\\wsl.localhost\Ubuntu\home\theo`.
pub fn to_windows(root: &str, linux: &str) -> PathBuf {
    PathBuf::from(format!("{root}{}", linux.replace('/', "\\")))
}

/// `\\wsl.localhost\Ubuntu\home\theo` → `/home/theo`.
pub fn to_linux(root: &str, p: &Path) -> String {
    p.to_string_lossy()[root.len()..].replace('\\', "/")
}

/// `.claude` and `.claude-*` dirs of every user of every distro.
#[cfg(windows)]
pub fn config_dirs() -> Vec<PathBuf> {
    let children = |d: PathBuf| std::fs::read_dir(d).into_iter().flatten().flatten().map(|e| e.path());
    distros()
        .into_iter()
        .map(|d| PathBuf::from(format!(r"\\wsl.localhost\{d}")))
        .flat_map(|root| children(root.join("home")).chain([root.join("root")]))
        .flat_map(children)
        .filter(|p| {
            p.is_dir()
                && p.file_name()
                    .is_some_and(|n| n == ".claude" || n.to_string_lossy().starts_with(".claude-"))
        })
        .collect()
}

#[cfg(not(windows))]
pub fn config_dirs() -> Vec<PathBuf> {
    vec![]
}

/// `wsl --list --quiet` prints UTF-16. No WSL, no distros.
#[cfg(windows)]
fn distros() -> Vec<String> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let Ok(out) = std::process::Command::new("wsl.exe")
        .args(["--list", "--quiet"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
    else {
        return vec![];
    };
    let utf16: Vec<u16> = out.stdout.as_chunks::<2>().0.iter().map(|&c| u16::from_le_bytes(c)).collect();
    String::from_utf16_lossy(&utf16)
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_roundtrip() {
        let p = Path::new(r"\\wsl.localhost\Ubuntu\home\theo\.claude");
        let root = root(p).unwrap();
        assert_eq!(root, r"\\wsl.localhost\Ubuntu");
        assert_eq!(distro(p).as_deref(), Some("Ubuntu"));
        assert_eq!(to_linux(&root, p), "/home/theo/.claude");
        assert_eq!(to_windows(&root, "/home/theo/.claude"), p);
        assert_eq!(super::root(Path::new(r"\\wsl$\Debian")).as_deref(), Some(r"\\wsl$\Debian"));
        assert_eq!(super::root(Path::new("/home/theo/.claude")), None);
    }
}
