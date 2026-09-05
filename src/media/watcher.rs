//! Server-owned inventory polling. A small periodic reconciliation also catches
//! missed filesystem notifications, offline edits and deletion after reattach.
use crate::events::AppEvent;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Sender as StopSender};
use std::time::{Duration, SystemTime};
use tokio::sync::mpsc::Sender;

type Fingerprint = (u64, SystemTime);
fn fingerprint(path: &Path) -> Option<Fingerprint> {
    let meta = std::fs::symlink_metadata(path).ok()?;
    meta.is_file().then(|| {
        (
            meta.len(),
            meta.modified().unwrap_or(SystemTime::UNIX_EPOCH),
        )
    })
}

pub struct MediaWatcher {
    stop: StopSender<()>,
}
impl Drop for MediaWatcher {
    fn drop(&mut self) {
        let _ = self.stop.send(());
    }
}

#[derive(Default)]
struct Collector {
    observed: HashMap<PathBuf, Fingerprint>,
    copied: HashMap<PathBuf, Fingerprint>,
}
impl Collector {
    fn new(root: &Path, media: &Path) -> Self {
        let baseline = super::files::scan(root)
            .into_iter()
            .filter(|p| !p.starts_with(media))
            .filter_map(|p| fingerprint(&p).map(|f| (p, f)))
            .collect();
        Self {
            observed: HashMap::new(),
            copied: baseline,
        }
    }
    fn tick(&mut self, root: &Path, media: &Path) {
        let current: HashMap<_, _> = super::files::scan(root)
            .into_iter()
            .filter(|p| !p.starts_with(media) && super::files::is_output(p))
            .filter_map(|p| fingerprint(&p).map(|f| (p, f)))
            .collect();
        for (path, signature) in &current {
            if self.copied.get(path) == Some(signature)
                || self.observed.get(path) != Some(signature)
            {
                continue;
            }
            match super::files::copy_file(path, &media.join("collected")) {
                Ok(_) => {
                    self.copied.insert(path.clone(), *signature);
                }
                Err(err) => tracing::warn!(?path, ?err, "media output copy deferred"),
            }
        }
        self.copied.retain(|path, _| current.contains_key(path));
        self.observed = current;
    }
}

pub fn start(
    root: &Path,
    workspace_id: String,
    event_tx: Sender<AppEvent>,
) -> std::io::Result<MediaWatcher> {
    let root = root.to_path_buf();
    let settings = super::MediaSettings::load(&root);
    let (stop, stopped) = mpsc::channel();
    std::thread::Builder::new()
        .name(format!("herdr-media-{workspace_id}"))
        .spawn(move || {
            let mut collector = if settings.collect_outputs {
                Collector::new(&root, &settings.directory)
            } else {
                Collector::default()
            };
            let mut previous = None;
            loop {
                if settings.collect_outputs {
                    collector.tick(&root, &settings.directory);
                }
                let inventory: Vec<_> = super::files::scan(&settings.directory)
                    .into_iter()
                    .filter_map(|p| fingerprint(&p).map(|f| (p, f)))
                    .collect();
                if previous.as_ref() != Some(&inventory) {
                    let event = AppEvent::MediaFilesObserved {
                        workspace_id: workspace_id.clone(),
                        directory: settings.directory.clone(),
                        paths: inventory.iter().map(|(p, _)| p.clone()).collect(),
                    };
                    match event_tx.try_send(event) {
                        Ok(()) => previous = Some(inventory),
                        Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => return,
                        Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => {}
                    }
                }
                match stopped.recv_timeout(Duration::from_secs(1)) {
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                    _ => return,
                }
            }
        })?;
    Ok(MediaWatcher { stop })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn output_is_copied_only_after_stable_scan_and_original_survives() {
        let root = std::env::temp_dir().join(format!(
            "herdr-collector-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let media = root.join("media");
        std::fs::create_dir_all(&media).unwrap();
        let mut collector = Collector::new(&root, &media);
        let source = root.join("report.pdf");
        std::fs::write(&source, b"first").unwrap();
        collector.tick(&root, &media);
        assert!(super::super::files::scan(&media).is_empty());
        std::fs::write(&source, b"complete document").unwrap();
        collector.tick(&root, &media);
        assert!(super::super::files::scan(&media).is_empty());
        collector.tick(&root, &media);
        let copies = super::super::files::scan(&media);
        assert_eq!(copies.len(), 1);
        assert_eq!(std::fs::read(&copies[0]).unwrap(), b"complete document");
        assert!(source.exists());
        collector.tick(&root, &media);
        assert_eq!(super::super::files::scan(&media).len(), 1);
        std::fs::write(root.join("AGENTS.md"), "instructions").unwrap();
        collector.tick(&root, &media);
        collector.tick(&root, &media);
        assert_eq!(super::super::files::scan(&media).len(), 1);
        std::fs::remove_dir_all(root).unwrap();
    }
}
