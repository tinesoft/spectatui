use std::path::Path;
use std::time::Duration;

use anyhow::{Context, Result};
use notify_debouncer_mini::{new_debouncer, DebouncedEventKind};
use tokio::sync::mpsc;

pub enum FsEvent {
    SpecsChanged,
    SpecifyChanged,
}

pub fn start_watcher(
    root: &Path,
    tx: mpsc::UnboundedSender<FsEvent>,
) -> Result<notify_debouncer_mini::Debouncer<notify::RecommendedWatcher>> {
    let specs_dir = root.join("specs");
    let specify_dir = root.join(".specify");

    let specs_prefix = specs_dir.clone();
    let specify_prefix = specify_dir.clone();

    let mut debouncer = new_debouncer(
        Duration::from_millis(500),
        move |res: Result<Vec<notify_debouncer_mini::DebouncedEvent>, notify::Error>| {
            if let Ok(events) = res {
                let mut specs_changed = false;
                let mut specify_changed = false;

                for event in &events {
                    if event.kind == DebouncedEventKind::Any {
                        if event.path.starts_with(&specs_prefix) {
                            specs_changed = true;
                        }
                        if event.path.starts_with(&specify_prefix) {
                            specify_changed = true;
                        }
                    }
                }

                if specs_changed {
                    let _ = tx.send(FsEvent::SpecsChanged);
                }
                if specify_changed {
                    let _ = tx.send(FsEvent::SpecifyChanged);
                }
            }
        },
    )
    .context("failed to create file watcher")?;

    if specs_dir.is_dir() {
        debouncer
            .watcher()
            .watch(&specs_dir, notify::RecursiveMode::Recursive)
            .context("failed to watch specs/")?;
    }

    if specify_dir.is_dir() {
        debouncer
            .watcher()
            .watch(&specify_dir, notify::RecursiveMode::Recursive)
            .context("failed to watch .specify/")?;
    }

    Ok(debouncer)
}
