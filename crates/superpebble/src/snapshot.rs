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
    let ts = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_millis());
    let dir = home().join(".superpebble/snapshots").join(ts.to_string());
    std::fs::create_dir_all(&dir)?;
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
