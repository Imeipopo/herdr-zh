//! Per-workspace media timeline: tracks files that have been dropped into a
//! workspace's pane or created inside its dedicated `media` directory,
//! so the TUI can render a chronological "what landed in this project" panel.
//!
//! Collection/configuration are shared server state. Only panel layout and
//! selection belong to the client presentation layer.

mod capture;
pub mod files;
mod settings;
pub use settings::{directory, expand_path, MediaSettings};
mod store;
pub mod watcher;

pub use capture::{capture_paths, quote_path};
pub use store::{MediaEntry, MediaIndex, MediaKind, MediaSource};

/// Dedicated project output directory, relative to a workspace's
/// `identity_cwd`. Dropped files are copied here and agents can write generated
/// images/documents here for them to appear in the timeline.
pub const DROPPED_FILES_DIR: &str = "media";

pub fn projects_root() -> std::path::PathBuf {
    #[cfg(test)]
    {
        std::env::temp_dir().join(format!("herdr-test-projects-{}", std::process::id()))
    }
    #[cfg(not(test))]
    {
        std::env::var_os("HOME")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(std::env::temp_dir)
            .join("Herdr Projects")
    }
}

pub fn next_project_dir() -> std::path::PathBuf {
    let root = projects_root();
    for number in 1.. {
        let candidate = root.join(format!("project-{number}"));
        if !candidate.exists() {
            return candidate;
        }
    }
    unreachable!()
}

pub fn project_dir_for_name(name: &str) -> std::path::PathBuf {
    let sanitized: String = name
        .trim()
        .chars()
        .map(|character| match character {
            '/' | '\\' | ':' | '\0' => '-',
            character => character,
        })
        .collect();
    let name = sanitized.trim_matches('.').trim();
    projects_root().join(if name.is_empty() { "project" } else { name })
}

pub fn initialize_project(project_dir: &std::path::Path) -> std::io::Result<()> {
    use std::io::Write as _;

    std::fs::create_dir_all(project_dir.join(DROPPED_FILES_DIR))?;
    let instructions = "\n<!-- herdr-media:start -->\n# Project images and documents\nSave final user-facing images/documents in the directory configured in `.herdr/media.json` (or `media/` if absent). Re-read this setting when starting a task. `$HERDR_MEDIA_DIR` is the launch-time fallback and may be stale after a setting change. Use a unique task subfolder to avoid conflicts with other agents. Preserve source files. Do not put code, logs, dependencies, audio or video here.\n\n請將最終圖片與文件存入 `.herdr/media.json` 指定的資料夾；無設定時使用 `media/`。每次任務使用不同子資料夾，避免覆蓋其他代理的成品。\n<!-- herdr-media:end -->\n";
    for name in ["AGENTS.md", "CLAUDE.md"] {
        let path = project_dir.join(name);
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        match options.open(path) {
            Ok(mut file) => file.write_all(instructions.as_bytes())?,
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {
                // Existing instructions are preserved byte-for-byte; add only our
                // bounded section. Refuse symlinks rather than touching another project.
                let path = project_dir.join(name);
                if !std::fs::symlink_metadata(&path)?.is_file() {
                    continue;
                }
                let existing = std::fs::read_to_string(&path)?;
                if !existing.contains("<!-- herdr-media:start -->") {
                    std::fs::OpenOptions::new()
                        .append(true)
                        .open(&path)?
                        .write_all(instructions.as_bytes())?;
                }
            }
            Err(err) => return Err(err),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
