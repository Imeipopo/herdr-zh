//! Filesystem operations shared by imports, migration and output collection.
use sha2::{Digest, Sha256};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

pub fn scan(root: &Path) -> Vec<PathBuf> {
    fn visit(root: &Path, paths: &mut Vec<PathBuf>, depth: usize) {
        if depth > 32 {
            return;
        }
        let Ok(entries) = std::fs::read_dir(root) else {
            return;
        };
        for entry in entries.flatten() {
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with('.')
                || matches!(
                    name.as_ref(),
                    "node_modules" | "target" | "vendor" | "build" | "dist" | "__pycache__"
                )
            {
                continue;
            }
            if kind.is_dir() {
                visit(&entry.path(), paths, depth + 1);
            } else if kind.is_file()
                && super::MediaKind::from_path(&entry.path()) != super::MediaKind::Other
            {
                paths.push(entry.path());
            }
        }
    }
    let mut paths = Vec::new();
    visit(root, &mut paths, 0);
    paths.sort();
    paths
}

pub fn is_output(path: &Path) -> bool {
    let name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_ascii_lowercase();
    ![
        "agents",
        "claude",
        "readme",
        "changelog",
        "license",
        "notice",
        "contributing",
    ]
    .iter()
    .any(|prefix| name == *prefix || name.starts_with(&format!("{prefix}.")))
}

/// Publish a complete immutable copy; identical contents/name reuse an existing
/// copy. A hard link from a private temporary file atomically refuses overwrite.
pub fn copy_file(source: &Path, directory: &Path) -> io::Result<PathBuf> {
    if !std::fs::symlink_metadata(source)?.is_file() {
        return Err(io::Error::other("Only regular files can be imported"));
    }
    std::fs::create_dir_all(directory)?;
    let mut input = std::fs::File::open(source)?;
    let before = input.metadata()?;
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let temporary = directory.join(format!(".import-{}-{stamp}", std::process::id()));
    let mut output = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)?;
    let result = (|| {
        let mut hash = Sha256::new();
        let mut buffer = [0; 65536];
        loop {
            let count = input.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            hash.update(&buffer[..count]);
            output.write_all(&buffer[..count])?;
        }
        output.sync_all()?;
        let after = input.metadata()?;
        if before.len() != after.len() || before.modified()? != after.modified()? {
            return Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                "File is still being written; retry shortly",
            ));
        }
        let digest = format!("{:x}", hash.finalize());
        let name = source.file_name().unwrap_or_default().to_string_lossy();
        // Keep common filenames readable, avoiding platform filename length limits.
        let mut start = 0;
        while name.len() - start > 180 {
            start += name[start..]
                .chars()
                .next()
                .map(char::len_utf8)
                .unwrap_or(1);
        }
        let name = &name[start..];
        let destination = directory.join(format!("{digest}-{name}"));
        match std::fs::hard_link(&temporary, &destination) {
            Ok(()) => Ok(destination),
            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => {
                let metadata = std::fs::symlink_metadata(&destination)?;
                if !metadata.is_file() {
                    return Err(io::Error::other("Import destination is not a regular file"));
                }
                let mut existing = std::fs::File::open(&destination)?;
                let mut existing_hash = Sha256::new();
                loop {
                    let count = existing.read(&mut buffer)?;
                    if count == 0 {
                        break;
                    }
                    existing_hash.update(&buffer[..count]);
                }
                if format!("{:x}", existing_hash.finalize()) == digest {
                    Ok(destination)
                } else {
                    Err(io::Error::new(
                        io::ErrorKind::AlreadyExists,
                        "A different file occupies the import destination",
                    ))
                }
            }
            Err(_) => {
                // FAT/exFAT and some network volumes cannot hard-link. Exclusive
                // creation still guarantees that no existing file is overwritten.
                let mut destination_file = std::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&destination)?;
                let result = (|| {
                    std::io::copy(&mut std::fs::File::open(&temporary)?, &mut destination_file)?;
                    destination_file.sync_all()
                })();
                drop(destination_file);
                if let Err(err) = result {
                    let _ = std::fs::remove_file(&destination);
                    return Err(err);
                }
                Ok(destination)
            }
        }
    })();
    drop(output);
    let _ = std::fs::remove_file(temporary);
    result
}
