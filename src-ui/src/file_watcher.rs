//! Repository file watching for event-driven workspace refresh.

use iced::{futures::SinkExt, stream, Subscription};
use log::{info, warn};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use tokio::sync::mpsc;
use tokio::time::{sleep_until, Duration, Instant as TokioInstant};

const WATCH_DEBOUNCE: Duration = Duration::from_millis(180);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryWatchEvent {
    pub repo_path: PathBuf,
}

pub fn subscription(repo_path: PathBuf) -> Subscription<RepositoryWatchEvent> {
    Subscription::run_with(repo_path, |repo_path| watch_repository(repo_path.as_path()))
}

fn watch_repository(repo_path: &Path) -> impl iced::futures::Stream<Item = RepositoryWatchEvent> {
    let repo_path = repo_path.to_path_buf();

    stream::channel(32, async move |mut output| {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let mut watcher = match RecommendedWatcher::new(
            move |result| {
                let _ = tx.send(result);
            },
            Config::default(),
        ) {
            Ok(watcher) => watcher,
            Err(error) => {
                warn!(
                    "Failed to create repository watcher for {}: {}",
                    repo_path.display(),
                    error
                );
                return;
            }
        };

        let mut watched_any = false;
        for path in watch_paths(&repo_path) {
            match watcher.watch(&path, RecursiveMode::Recursive) {
                Ok(()) => watched_any = true,
                Err(error) => warn!(
                    "Failed to watch {} for repository {}: {}",
                    path.display(),
                    repo_path.display(),
                    error
                ),
            }
        }

        if !watched_any {
            warn!(
                "Repository watcher did not attach to any paths for {}",
                repo_path.display()
            );
            return;
        }

        info!("Watching repository changes for {}", repo_path.display());

        let mut pending_deadline: Option<TokioInstant> = None;

        loop {
            if let Some(deadline) = pending_deadline {
                tokio::select! {
                    maybe_result = rx.recv() => {
                        let Some(result) = maybe_result else {
                            break;
                        };

                        if should_schedule_refresh(&result) {
                            pending_deadline = Some(TokioInstant::now() + WATCH_DEBOUNCE);
                        }
                    }
                    _ = sleep_until(deadline) => {
                        pending_deadline = None;

                        if output
                            .send(RepositoryWatchEvent {
                                repo_path: repo_path.clone(),
                            })
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                }
            } else {
                let Some(result) = rx.recv().await else {
                    break;
                };

                if should_schedule_refresh(&result) {
                    pending_deadline = Some(TokioInstant::now() + WATCH_DEBOUNCE);
                }
            }
        }
    })
}

fn should_schedule_refresh(result: &Result<Event, notify::Error>) -> bool {
    match result {
        Ok(event) => is_relevant_event(event),
        Err(error) => {
            warn!("Repository watcher error: {}", error);
            false
        }
    }
}

fn is_relevant_event(event: &Event) -> bool {
    matches!(
        event.kind,
        EventKind::Any | EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
    ) && !event.paths.is_empty()
}

fn watch_paths(repo_path: &Path) -> Vec<PathBuf> {
    let mut paths = vec![repo_path.to_path_buf()];

    if let Some(git_dir) = resolve_git_dir(repo_path) {
        if !git_dir.starts_with(repo_path) && !paths.iter().any(|candidate| candidate == &git_dir) {
            paths.push(git_dir);
        }
    }

    paths
}

fn resolve_git_dir(repo_path: &Path) -> Option<PathBuf> {
    let dot_git = repo_path.join(".git");

    if dot_git.is_dir() {
        return Some(dot_git);
    }

    if !dot_git.is_file() {
        return None;
    }

    let contents = std::fs::read_to_string(&dot_git).ok()?;
    let gitdir = contents.strip_prefix("gitdir:")?.trim();
    let path = PathBuf::from(gitdir);

    Some(if path.is_absolute() {
        path
    } else {
        repo_path.join(path)
    })
}
