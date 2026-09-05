use std::path::{Path, PathBuf};
use std::time::SystemTime;

use super::store::{MediaEntry, MediaIndex, MediaKind, MediaSource};
#[cfg(test)]
use super::DROPPED_FILES_DIR;

/// Split a pasted string into shell-style tokens, honoring single and double
/// quotes the way terminal emulators quote dropped file paths (e.g.
/// `'/Users/me/a.png' '/Users/me/b.png'`). Best-effort: unmatched quotes just
/// fall back to treating the rest of the string as one token.
fn shell_tokenize(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut quote = None;
    let mut chars = text.chars();
    while let Some(ch) = chars.next() {
        match (quote, ch) {
            (Some(q), c) if q == c => quote = None,
            (Some('\''), c) => current.push(c),
            (_, '\\') => {
                if let Some(next) = chars.next() {
                    current.push(next);
                }
            }
            (Some(_), c) => current.push(c),
            (None, '\'' | '"') => quote = Some(ch),
            (None, c) if c.is_whitespace() => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            (_, c) => current.push(c),
        }
    }
    if quote.is_some() {
        return Vec::new();
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

pub fn quote_path(path: &Path) -> String {
    format!("'{}'", path.to_string_lossy().replace('\'', "'\\''"))
}

/// Only an entire paste consisting of supported existing file paths is an
/// import. Ordinary prose/commands pass through untouched. Failed imports are
/// returned to the caller and must never be silently forwarded as successful.
pub fn capture_paths(
    text: &str,
    identity_cwd: &Path,
    workspace_id: &str,
    index: &mut MediaIndex,
) -> std::io::Result<String> {
    let tokens = shell_tokenize(text);
    if tokens.is_empty()
        || tokens.iter().any(|token| {
            let path = Path::new(token);
            !path.is_absolute() || !path.is_file() || MediaKind::from_path(path) == MediaKind::Other
        })
    {
        return Ok(text.to_owned());
    }
    let destination = super::directory(identity_cwd);
    let mut pasted = Vec::new();
    for token in tokens {
        let source = PathBuf::from(token);
        let source = source.canonicalize()?;
        let dest = if source.starts_with(
            destination
                .canonicalize()
                .unwrap_or_else(|_| destination.clone()),
        ) {
            source
        } else {
            super::files::copy_file(&source, &destination)?
        };
        index.record(
            workspace_id,
            MediaEntry {
                kind: MediaKind::from_path(&dest),
                path: dest.clone(),
                added_at: SystemTime::now(),
                source: MediaSource::Dropped,
            },
        );
        pasted.push(quote_path(&dest));
    }
    index.save(workspace_id, identity_cwd);
    Ok(pasted.join(" "))
}

#[cfg(test)]
pub fn maybe_capture_dropped_paths(text: &str, root: &Path, id: &str, index: &mut MediaIndex) {
    let _ = capture_paths(text, root, id, index);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenizes_quoted_paths() {
        let tokens = shell_tokenize("'/Users/me/a b.png' '/Users/me/c.png'");
        assert_eq!(tokens, vec!["/Users/me/a b.png", "/Users/me/c.png"]);
    }

    #[test]
    fn tokenizes_unquoted_whitespace_separated() {
        let tokens = shell_tokenize("/a.png /b.png");
        assert_eq!(tokens, vec!["/a.png", "/b.png"]);
    }

    #[test]
    fn ignores_non_path_text() {
        let dir = std::env::temp_dir().join(format!(
            "herdr-capture-test-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        std::env::set_var("XDG_STATE_HOME", &dir);

        let mut index = MediaIndex::new();
        maybe_capture_dropped_paths("hello world, not a path", &dir, "w1", &mut index);
        assert!(index.entries_for("w1").is_empty());

        let _ = std::fs::remove_dir_all(&dir);
        std::env::remove_var("XDG_STATE_HOME");
    }

    #[test]
    fn copies_dropped_file_and_records_entry() {
        let base = std::env::temp_dir().join(format!(
            "herdr-capture-test2-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let project_dir = base.join("project");
        std::fs::create_dir_all(&project_dir).unwrap();
        std::env::set_var("XDG_STATE_HOME", base.join("state"));

        let src_file = base.join("dropped.png");
        std::fs::write(&src_file, b"fake png bytes").unwrap();

        let mut index = MediaIndex::new();
        let paste_text = format!("'{}'", src_file.display());
        maybe_capture_dropped_paths(&paste_text, &project_dir, "w1", &mut index);

        let entries = index.entries_for("w1");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].kind, MediaKind::Image);
        assert_eq!(entries[0].source, MediaSource::Dropped);
        assert!(entries[0]
            .path
            .starts_with(project_dir.join(DROPPED_FILES_DIR)));
        assert!(entries[0].path.exists());

        let _ = std::fs::remove_dir_all(&base);
        std::env::remove_var("XDG_STATE_HOME");
    }
}
