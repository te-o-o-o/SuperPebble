//! Every write goes through here: copy the files to `~/.superpebble/snapshots/<ts>/` first,
//! then replace them atomically (temp file + rename).

use crate::scan::home;
use serde_json::json;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Copies `files` (those that exist) into a new snapshot dir with a `manifest.json` mapping
/// each copy back to its original path. Returns the snapshot dir.
pub fn save(files: &[&Path], reason: &str) -> std::io::Result<PathBuf> {
    let mut ts = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_millis());
    std::fs::create_dir_all(root())?;
    // Two snapshots within the same millisecond must not share a dir.
    let dir = loop {
        let dir = root().join(ts.to_string());
        match std::fs::create_dir(&dir) {
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => ts += 1,
            res => break res.map(|_| dir)?,
        }
    };
    let mut entries = vec![];
    for (i, f) in files.iter().enumerate().filter(|(_, f)| f.is_file()) {
        let copy = format!("{i}-{}", f.file_name().unwrap_or_default().to_string_lossy());
        std::fs::copy(f, dir.join(&copy))?;
        entries.push(json!({ "original": f, "copy": copy }));
    }
    let manifest = json!({ "reason": reason, "files": entries });
    std::fs::write(dir.join("manifest.json"), serde_json::to_string_pretty(&manifest)?)?;
    Ok(dir)
}

fn root() -> PathBuf {
    home().join(".superpebble/snapshots")
}

#[derive(serde::Serialize)]
pub struct Snapshot {
    /// The dir name: creation time in Unix milliseconds.
    pub id: String,
    pub reason: String,
    pub files: Vec<PathBuf>,
}

/// The 20 latest snapshots, newest first.
pub fn list() -> Vec<Snapshot> {
    let mut ids: Vec<u128> = std::fs::read_dir(root())
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| e.file_name().to_str()?.parse().ok())
        .collect();
    ids.sort_unstable_by(|a, b| b.cmp(a));
    ids.iter()
        .take(20)
        .filter_map(|id| {
            let m = manifest(&id.to_string()).ok()?;
            let files = m["files"]
                .as_array()?
                .iter()
                .filter_map(|f| f["original"].as_str().map(PathBuf::from))
                .collect();
            Some(Snapshot {
                id: id.to_string(),
                reason: m["reason"].as_str().unwrap_or_default().to_string(),
                files,
            })
        })
        .collect()
}

fn manifest(id: &str) -> Result<serde_json::Value, String> {
    // A number only: the id must not walk out of the snapshots dir.
    if id.is_empty() || !id.bytes().all(|b| b.is_ascii_digit()) {
        return Err("Unknown snapshot.".into());
    }
    let text = std::fs::read_to_string(root().join(id).join("manifest.json")).map_err(|_| "Unknown snapshot.")?;
    serde_json::from_str(&text).map_err(|e| e.to_string())
}

/// Puts the files of snapshot `id` back, after a snapshot of their current state, so a restore
/// can be undone too. Files created since the snapshot are left alone.
pub fn restore(id: &str) -> Result<(), String> {
    let m = manifest(id)?;
    let dir = root().join(id);
    let pairs: Vec<(PathBuf, PathBuf)> = m["files"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|f| Some((PathBuf::from(f["original"].as_str()?), dir.join(f["copy"].as_str()?))))
        .collect();
    let originals: Vec<&Path> = pairs.iter().map(|(o, _)| o.as_path()).collect();
    save(&originals, &format!("before restoring {id}")).map_err(|e| e.to_string())?;
    for (original, copy) in &pairs {
        let text = std::fs::read_to_string(copy).map_err(|e| e.to_string())?;
        write_atomic(original, &text).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Writes through symlinks (dotfile managers link `~/.zshrc`), keeping the target's permissions.
pub fn write_atomic(path: &Path, content: &str) -> std::io::Result<()> {
    let target = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let dir = target.parent().unwrap_or(Path::new("."));
    let tmp = dir.join(format!(
        ".{}.superpebble-tmp",
        target.file_name().unwrap_or_default().to_string_lossy()
    ));
    let mut f = std::fs::File::create(&tmp)?;
    f.write_all(content.as_bytes())?;
    f.sync_all()?;
    if let Ok(meta) = std::fs::metadata(&target) {
        std::fs::set_permissions(&tmp, meta.permissions())?;
    }
    std::fs::rename(&tmp, &target)
}

#[cfg(all(test, unix))]
mod tests {
    use super::write_atomic;

    #[test]
    fn writes_through_symlink() {
        let dir = std::env::temp_dir().join(format!("sp-atomic-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let (real, link) = (dir.join("real"), dir.join("link"));
        std::fs::write(&real, "old").unwrap();
        let _ = std::fs::remove_file(&link);
        std::os::unix::fs::symlink(&real, &link).unwrap();

        write_atomic(&link, "new").unwrap();
        assert!(std::fs::symlink_metadata(&link).unwrap().file_type().is_symlink());
        assert_eq!(std::fs::read_to_string(&real).unwrap(), "new");
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
