use super::*;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "herdr-media-regression-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&path).unwrap();
        Self(path.canonicalize().unwrap())
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn imports_escaped_chinese_paths_preserves_prose_and_deduplicates() {
    let root = Temp::new();
    let project = root.0.join("project");
    std::fs::create_dir(&project).unwrap();
    let source = root.0.join("中文 photo's.jpg");
    std::fs::write(&source, b"image one").unwrap();
    let paste = source
        .display()
        .to_string()
        .replace(' ', "\\ ")
        .replace('\'', "\\'");
    let mut index = MediaIndex::new();
    let output = capture_paths(&paste, &project, "w1", &mut index).unwrap();
    let copied = index.entries_for("w1")[0].path.clone();
    assert_eq!(output, quote_path(&copied));
    assert!(copied.starts_with(project.join("media")));
    assert_eq!(std::fs::read(&source).unwrap(), b"image one");
    capture_paths(&paste, &project, "w1", &mut index).unwrap();
    assert_eq!(index.entries_for("w1").len(), 1);
    let prose = format!("please inspect {}", quote_path(&source));
    assert_eq!(
        capture_paths(&prose, &project, "w1", &mut index).unwrap(),
        prose
    );
    assert_eq!(
        capture_paths(&output, &project, "w1", &mut index).unwrap(),
        output
    );
    let second_project = root.0.join("another project");
    capture_paths(&paste, &second_project, "w2", &mut index).unwrap();
    assert!(index.entries_for("w2")[0].path.starts_with(&second_project));
}

#[test]
fn same_name_different_contents_never_overwrite_and_unicode_name_fits() {
    let root = Temp::new();
    let source = root.0.join(format!("{}.docx", "報告".repeat(30)));
    std::fs::write(&source, b"one").unwrap();
    let first = files::copy_file(&source, &root.0.join("media")).unwrap();
    std::fs::write(&source, b"two").unwrap();
    let second = files::copy_file(&source, &root.0.join("media")).unwrap();
    assert_ne!(first, second);
    assert_eq!(std::fs::read(&first).unwrap(), b"one");
    assert_eq!(std::fs::read(&second).unwrap(), b"two");
    assert_eq!(std::fs::read(&source).unwrap(), b"two");
}

#[test]
fn disk_inventory_survives_restart_tracks_updates_deletions_and_filters() {
    let root = Temp::new();
    let media = root.0.join("media");
    std::fs::create_dir(&media).unwrap();
    for name in [
        "image.jpg",
        "report.docx",
        "report.pdf",
        "notes.txt",
        "source.rs",
        "video.mp4",
        "audio.mp3",
    ] {
        std::fs::write(media.join(name), b"initial").unwrap();
    }
    let mut index = MediaIndex::new();
    index.reconcile("w1", files::scan(&media));
    assert_eq!(index.entries_for("w1").len(), 4);
    std::fs::remove_file(media.join("report.pdf")).unwrap();
    std::fs::write(media.join("report.docx"), b"updated").unwrap();
    index.reconcile("w1", files::scan(&media));
    assert_eq!(index.entries_for("w1").len(), 3);
    assert!(index
        .entries_for("w1")
        .iter()
        .all(|entry| entry.path.exists()));
    let mut restarted = MediaIndex::new();
    restarted.reconcile("w1", files::scan(&media));
    assert_eq!(restarted.entries_for("w1").len(), 3);
}

#[test]
fn configuration_migration_preserves_files_and_existing_instructions() {
    let root = Temp::new();
    std::fs::write(root.0.join("AGENTS.md"), "Keep my rules.\n").unwrap();
    initialize_project(&root.0).unwrap();
    initialize_project(&root.0).unwrap();
    let instructions = std::fs::read_to_string(root.0.join("AGENTS.md")).unwrap();
    assert!(instructions.starts_with("Keep my rules.\n"));
    assert_eq!(
        instructions.matches("<!-- herdr-media:start -->").count(),
        1
    );
    let original = root.0.join("media/existing.pdf");
    std::fs::write(&original, b"document").unwrap();
    let settings = MediaSettings {
        directory: root.0.join("attachments"),
        collect_outputs: true,
    };
    settings.save(&root.0, true).unwrap();
    assert_eq!(MediaSettings::load(&root.0), settings);
    assert!(original.exists());
    assert_eq!(files::scan(&settings.directory).len(), 1);
    assert!(MediaSettings {
        directory: root.0.clone(),
        collect_outputs: false
    }
    .save(&root.0, false)
    .is_err());
}

#[test]
fn scanner_excludes_dependencies_and_hidden_files() {
    let root = Temp::new();
    for folder in ["node_modules", "target", ".git", "dist"] {
        std::fs::create_dir(root.0.join(folder)).unwrap();
        std::fs::write(root.0.join(folder).join("image.jpg"), b"image").unwrap();
    }
    std::fs::write(root.0.join(".partial.jpg"), b"partial").unwrap();
    assert!(files::scan(&root.0).is_empty());
    assert!(!files::is_output(&root.0.join("AGENTS.md")));
    assert!(!files::is_output(&root.0.join("README.zh-TW.md")));
    assert!(files::is_output(&root.0.join("report.pdf")));
}

#[cfg(unix)]
#[test]
fn scanner_does_not_follow_symlinks() {
    let root = Temp::new();
    let external = Temp::new();
    std::fs::write(external.0.join("report.pdf"), b"private").unwrap();
    std::os::unix::fs::symlink(&external.0, root.0.join("external")).unwrap();
    std::os::unix::fs::symlink(&root.0, root.0.join("cycle")).unwrap();
    assert!(files::scan(&root.0).is_empty());
}
