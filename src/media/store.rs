use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use serde::{Deserialize, Serialize};
use tracing::warn;

/// How a `MediaEntry` was recorded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MediaSource {
    /// Copied out of a paste event that looked like a dropped file path.
    Dropped,
    /// Observed via the workspace directory watcher.
    Watched,
}

/// Coarse file classification, derived from the extension only (no content
/// sniffing / decoding).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MediaKind {
    Image,
    Document,
    Other,
}

impl MediaKind {
    pub fn from_path(path: &Path) -> Self {
        const IMAGE_EXTS: &[&str] = &["png", "jpg", "jpeg", "gif", "webp", "heic"];
        const DOCUMENT_EXTS: &[&str] = &[
            "pdf", "txt", "md", "doc", "docx", "xls", "xlsx", "csv", "ppt", "pptx",
        ];
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(str::to_ascii_lowercase);
        match extension.as_deref() {
            Some(ext) if IMAGE_EXTS.contains(&ext) => MediaKind::Image,
            Some(ext) if DOCUMENT_EXTS.contains(&ext) => MediaKind::Document,
            _ => MediaKind::Other,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaEntry {
    /// Where the file actually lives on disk right now (a copy under
    /// `DROPPED_FILES_DIR` for `Dropped` entries, the original path in place
    /// for `Watched` entries).
    pub path: PathBuf,
    pub added_at: SystemTime,
    pub kind: MediaKind,
    pub source: MediaSource,
}

/// In-memory index of media entries, keyed by `Workspace.id`. Pure data:
/// no PTY/watcher access here, so it is directly unit-testable.
#[derive(Debug, Default)]
pub struct MediaIndex {
    by_workspace: HashMap<String, Vec<MediaEntry>>,
    directories: HashMap<String, PathBuf>,
}

impl MediaIndex {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record an entry for a workspace, keeping entries sorted newest-first.
    /// Silently ignores an exact duplicate path already recorded for this
    /// workspace (the watcher can fire multiple events for one file).
    pub fn record(&mut self, workspace_id: &str, entry: MediaEntry) {
        let entries = self
            .by_workspace
            .entry(workspace_id.to_string())
            .or_default();
        let mut entry = entry;
        if let Some(position) = entries.iter().position(|e| e.path == entry.path) {
            let old = entries.remove(position);
            entry.source = old.source;
        }
        let pos = entries.partition_point(|e| e.added_at > entry.added_at);
        entries.insert(pos, entry);
    }

    pub fn entries_for(&self, workspace_id: &str) -> &[MediaEntry] {
        self.by_workspace
            .get(workspace_id)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// On-disk key is a hash of the *project folder's* canonical path, not
    /// the workspace id: `Workspace.id` is only unique within one running
    /// server process (a simple per-process counter), so two independent
    /// sessions can both have a "w1" whose histories must not collide. Two
    /// workspaces that really do point at the same folder sharing history
    /// is correct, not a bug.
    /// Reconcile the full on-disk inventory, including additions, updates and removals.
    pub fn reconcile(&mut self, workspace_id: &str, paths: Vec<PathBuf>) {
        let previous = self.by_workspace.remove(workspace_id).unwrap_or_default();
        for path in paths {
            let Ok(metadata) = std::fs::symlink_metadata(&path) else {
                continue;
            };
            if !metadata.is_file() || MediaKind::from_path(&path) == MediaKind::Other {
                continue;
            }
            let source = previous
                .iter()
                .find(|e| e.path == path)
                .map(|e| e.source)
                .unwrap_or(MediaSource::Watched);
            self.record(
                workspace_id,
                MediaEntry {
                    kind: MediaKind::from_path(&path),
                    path,
                    added_at: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
                    source,
                },
            );
        }
    }

    #[cfg(not(test))]
    fn path_key(identity_cwd: &Path) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let canonical = identity_cwd
            .canonicalize()
            .unwrap_or_else(|_| identity_cwd.to_path_buf());
        let mut hasher = DefaultHasher::new();
        canonical.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }

    #[cfg(test)]
    fn index_path(identity_cwd: &Path) -> PathBuf {
        identity_cwd.join(".test-media-index.json")
    }

    #[cfg(not(test))]
    fn index_path(identity_cwd: &Path) -> PathBuf {
        crate::config::state_dir()
            .join("media")
            .join(format!("{}.json", Self::path_key(identity_cwd)))
    }

    /// Load a single workspace's persisted entries into this index, merging
    /// with (not replacing) anything already recorded in memory.
    pub fn directory_for(&self, workspace_id: &str) -> Option<&Path> {
        self.directories.get(workspace_id).map(PathBuf::as_path)
    }

    pub fn load(&mut self, workspace_id: &str, identity_cwd: &Path) {
        self.directories
            .insert(workspace_id.to_owned(), super::directory(identity_cwd));
        let path = Self::index_path(identity_cwd);
        let bytes = match std::fs::read(&path) {
            Ok(bytes) => bytes,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return,
            Err(err) => {
                warn!(?err, ?path, "failed to read media index");
                return;
            }
        };
        match serde_json::from_slice::<Vec<MediaEntry>>(&bytes) {
            Ok(entries) => {
                let media_dir = super::directory(identity_cwd);
                for mut entry in entries {
                    // Older builds watched every file in the workspace. Keep
                    // explicitly dropped files, but discard non-attachment
                    // watcher entries so legacy indexes do not refill the
                    // panel with source code and build artifacts. Reclassify
                    // paths so indexes written before the richer media kinds
                    // were added migrate automatically.
                    entry.kind = MediaKind::from_path(&entry.path);
                    if !entry.path.starts_with(&media_dir) {
                        continue;
                    }
                    if !entry.path.is_file() {
                        continue;
                    }
                    if entry.source == MediaSource::Watched && entry.kind == MediaKind::Other {
                        continue;
                    }
                    self.record(workspace_id, entry);
                }
            }
            Err(err) => warn!(?err, ?path, "failed to parse media index"),
        }
    }

    pub fn save(&self, workspace_id: &str, identity_cwd: &Path) {
        let path = Self::index_path(identity_cwd);
        let Some(parent) = path.parent() else { return };
        if let Err(err) = std::fs::create_dir_all(parent) {
            warn!(?err, ?parent, "failed to create media index directory");
            return;
        }
        let entries = self.entries_for(workspace_id);
        match serde_json::to_vec_pretty(entries) {
            Ok(bytes) => {
                if let Err(err) = std::fs::write(&path, bytes) {
                    warn!(?err, ?path, "failed to write media index");
                }
            }
            Err(err) => warn!(?err, "failed to serialize media index"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn entry(path: &str, added_at: SystemTime, source: MediaSource) -> MediaEntry {
        MediaEntry {
            path: PathBuf::from(path),
            added_at,
            kind: MediaKind::from_path(Path::new(path)),
            source,
        }
    }

    #[test]
    fn kind_from_extension() {
        assert_eq!(MediaKind::from_path(Path::new("a.PNG")), MediaKind::Image);
        assert_eq!(
            MediaKind::from_path(Path::new("brief.pdf")),
            MediaKind::Document
        );
        assert_eq!(
            MediaKind::from_path(Path::new("voice.m4a")),
            MediaKind::Other
        );
        assert_eq!(
            MediaKind::from_path(Path::new("demo.mov")),
            MediaKind::Other
        );
        assert_eq!(MediaKind::from_path(Path::new("a.rs")), MediaKind::Other);
        assert_eq!(MediaKind::from_path(Path::new("a")), MediaKind::Other);
    }

    #[test]
    fn record_sorts_newest_first() {
        let mut index = MediaIndex::new();
        let t0 = SystemTime::UNIX_EPOCH;
        let t1 = t0 + Duration::from_secs(10);
        let t2 = t0 + Duration::from_secs(20);
        index.record("w1", entry("/a", t0, MediaSource::Watched));
        index.record("w1", entry("/c", t2, MediaSource::Watched));
        index.record("w1", entry("/b", t1, MediaSource::Watched));

        let paths: Vec<_> = index
            .entries_for("w1")
            .iter()
            .map(|e| e.path.to_string_lossy().to_string())
            .collect();
        assert_eq!(paths, vec!["/c", "/b", "/a"]);
    }

    #[test]
    fn record_dedupes_by_path() {
        let mut index = MediaIndex::new();
        let t0 = SystemTime::UNIX_EPOCH;
        index.record("w1", entry("/a", t0, MediaSource::Watched));
        index.record(
            "w1",
            entry("/a", t0 + Duration::from_secs(5), MediaSource::Dropped),
        );
        assert_eq!(index.entries_for("w1").len(), 1);
    }

    #[test]
    fn entries_for_unknown_workspace_is_empty() {
        let index = MediaIndex::new();
        assert!(index.entries_for("nope").is_empty());
    }

    #[test]
    fn save_and_load_round_trip() {
        let dir = std::env::temp_dir().join(format!(
            "herdr-media-test-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::env::set_var("XDG_STATE_HOME", &dir);
        let project_dir = dir.join("project");

        let mut index = MediaIndex::new();
        let t0 = SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000);
        let media_path = project_dir
            .join(crate::media::DROPPED_FILES_DIR)
            .join("photo.png");
        std::fs::create_dir_all(media_path.parent().unwrap()).unwrap();
        std::fs::write(&media_path, b"png").unwrap();
        index.record(
            "wtest",
            MediaEntry {
                path: media_path.clone(),
                added_at: t0,
                kind: MediaKind::Image,
                source: MediaSource::Dropped,
            },
        );
        index.save("wtest", &project_dir);

        let mut reloaded = MediaIndex::new();
        reloaded.load("wtest", &project_dir);
        assert_eq!(reloaded.entries_for("wtest").len(), 1);
        assert_eq!(reloaded.entries_for("wtest")[0].path, media_path);

        let _ = std::fs::remove_dir_all(&dir);
        std::env::remove_var("XDG_STATE_HOME");
    }
}
