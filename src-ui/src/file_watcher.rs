//! File watcher for repository changes
//!
//! Uses notify crate to monitor file system changes in git repositories

use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use log::{info, error};

/// File change event
#[derive(Debug, Clone)]
pub enum FileChange {
    Created(PathBuf),
    Modified(PathBuf),
    Deleted(PathBuf),
    Unknown(PathBuf),
}

/// File watcher that monitors a directory for changes
pub struct FileWatcher {
    watcher: RecommendedWatcher,
    receiver: Receiver<Result<Event, notify::Error>>,
}

impl FileWatcher {
    /// Create a new file watcher for the given path
    pub fn new(path: &Path) -> Result<Self, notify::Error> {
        let (tx, rx) = channel();

        let mut watcher = RecommendedWatcher::new(
            move |res| {
                let _ = tx.send(res);
            },
            Config::default(),
        )?;

        watcher.watch(path, RecursiveMode::Recursive)?;

        info!("File watcher started for: {:?}", path);

        Ok(Self {
            watcher,
            receiver: rx,
        })
    }

    /// Check for file changes
    pub fn poll_changes(&self) -> Vec<FileChange> {
        let mut changes = Vec::new();

        while let Ok(result) = self.receiver.try_recv() {
            match result {
                Ok(event) => {
                    for path in event.paths {
                        let change = match event.kind {
                            notify::EventKind::Create(_) => FileChange::Created(path),
                            notify::EventKind::Modify(_) => FileChange::Modified(path),
                            notify::EventKind::Remove(_) => FileChange::Deleted(path),
                            _ => FileChange::Unknown(path),
                        };
                        changes.push(change);
                    }
                }
                Err(e) => {
                    error!("File watcher error: {}", e);
                }
            }
        }

        changes
    }
}

/// Start a background file watching task
pub fn start_watcher(path: PathBuf) -> Sender<FileChange> {
    let (tx, rx) = channel();

    thread::spawn(move || {
        match FileWatcher::new(&path) {
            Ok(watcher) => {
                info!("Background file watcher started");
                loop {
                    let changes = watcher.poll_changes();
                    for change in changes {
                        let _ = tx.send(change);
                    }
                    thread::sleep(std::time::Duration::from_millis(100));
                }
            }
            Err(e) => {
                error!("Failed to start file watcher: {}", e);
            }
        }
    });

    tx
}
