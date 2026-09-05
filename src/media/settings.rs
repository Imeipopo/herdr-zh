use serde::{Deserialize, Serialize};
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
pub struct MediaSettings {
    pub directory: PathBuf,
    #[serde(default)]
    pub collect_outputs: bool,
}

impl MediaSettings {
    pub fn load(root: &Path) -> Self {
        let path = root.join(".herdr/media.json");
        if let Ok(bytes) = std::fs::read(path) {
            if let Ok(settings) = serde_json::from_slice(&bytes) {
                return settings;
            }
        }
        Self {
            directory: root.join(super::DROPPED_FILES_DIR),
            collect_outputs: false,
        }
    }

    pub fn save(&self, root: &Path, copy_existing: bool) -> io::Result<()> {
        let previous = Self::load(root);
        std::fs::create_dir_all(&self.directory)?;
        let destination = self.directory.canonicalize()?;
        if root.canonicalize()?.starts_with(&destination) {
            return Err(io::Error::other(
                "Media folder cannot be the project root or one of its parent folders",
            ));
        }
        if copy_existing && previous.directory != self.directory {
            for path in super::files::scan(&previous.directory) {
                if !path.starts_with(&self.directory) {
                    super::files::copy_file(&path, &self.directory)?;
                }
            }
        }
        let folder = root.join(".herdr");
        std::fs::create_dir_all(&folder)?;
        let settings = Self {
            directory: destination,
            collect_outputs: self.collect_outputs,
        };
        let temporary = folder.join(format!("media-{}.tmp", std::process::id()));
        std::fs::write(&temporary, serde_json::to_vec_pretty(&settings)?)?;
        std::fs::rename(temporary, folder.join("media.json"))?;
        Ok(())
    }
}

pub fn directory(root: &Path) -> PathBuf {
    MediaSettings::load(root).directory
}

pub fn expand_path(text: &str) -> io::Result<PathBuf> {
    let text = text.trim();
    let path = if text == "~" || text.starts_with("~/") {
        super::projects_root()
            .parent()
            .unwrap_or(Path::new("/"))
            .join(text.trim_start_matches('~').trim_start_matches('/'))
    } else {
        PathBuf::from(text)
    };
    if !path.is_absolute() {
        return Err(io::Error::other("Please enter an absolute path (or ~/...)"));
    }
    if path
        .components()
        .any(|part| matches!(part, std::path::Component::ParentDir))
    {
        return Err(io::Error::other("Folder path must not contain .."));
    }
    Ok(path)
}
