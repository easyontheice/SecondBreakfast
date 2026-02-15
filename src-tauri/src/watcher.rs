use crate::errors::{AppError, AppResult};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

pub type DebouncedAction = Arc<dyn Fn() + Send + Sync + 'static>;

#[derive(Debug)]
pub struct WatcherController {
    pub running: bool,
    stop_tx: Option<Sender<()>>,
    handle: Option<JoinHandle<()>>,
}

impl Default for WatcherController {
    fn default() -> Self {
        Self {
            running: false,
            stop_tx: None,
            handle: None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WatcherStatus {
    pub running: bool,
    pub sort_root: String,
}

pub fn start_watcher(
    controller: &Arc<Mutex<WatcherController>>,
    sort_root: PathBuf,
    debounce: Duration,
    action: DebouncedAction,
) -> AppResult<()> {
    let mut guard = controller.lock()?;
    if guard.running {
        return Ok(());
    }

    let (stop_tx, stop_rx) = mpsc::channel::<()>();
    let (startup_tx, startup_rx) = mpsc::channel::<Result<(), notify::Error>>();

    let handle = thread::spawn(move || {
        let (event_tx, event_rx) = mpsc::channel::<Result<Event, notify::Error>>();

        let mut watcher = match RecommendedWatcher::new(
            move |res| {
                let _ = event_tx.send(res);
            },
            notify::Config::default(),
        ) {
            Ok(w) => w,
            Err(err) => {
                let _ = startup_tx.send(Err(err));
                return;
            }
        };

        if let Err(err) = watcher.watch(&sort_root, RecursiveMode::Recursive) {
            let _ = startup_tx.send(Err(err));
            return;
        }

        let _ = startup_tx.send(Ok(()));
        run_loop(event_rx, stop_rx, debounce, action);
    });

    match startup_rx.recv_timeout(Duration::from_secs(5)) {
        Ok(Ok(())) => {
            guard.running = true;
            guard.stop_tx = Some(stop_tx);
            guard.handle = Some(handle);
            Ok(())
        }
        Ok(Err(err)) => {
            let _ = handle.join();
            Err(AppError::Notify(err))
        }
        Err(err) => {
            let _ = handle.join();
            Err(AppError::State(format!("watcher failed to start: {}", err)))
        }
    }
}

pub fn stop_watcher(controller: &Arc<Mutex<WatcherController>>) -> AppResult<()> {
    let mut guard = controller.lock()?;
    if !guard.running {
        return Ok(());
    }

    if let Some(tx) = guard.stop_tx.take() {
        let _ = tx.send(());
    }

    if let Some(handle) = guard.handle.take() {
        let _ = handle.join();
    }

    guard.running = false;
    Ok(())
}

fn run_loop(
    event_rx: Receiver<Result<Event, notify::Error>>,
    stop_rx: Receiver<()>,
    debounce: Duration,
    action: DebouncedAction,
) {
    let mut pending_at: Option<Instant> = None;

    loop {
        if stop_rx.try_recv().is_ok() {
            break;
        }

        match event_rx.recv_timeout(Duration::from_millis(200)) {
            Ok(Ok(event)) => {
                if is_sorting_relevant(&event.kind) {
                    pending_at = Some(Instant::now());
                }
            }
            Ok(Err(_)) => {}
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }

        if let Some(started) = pending_at {
            if started.elapsed() >= debounce {
                pending_at = None;
                action();
            }
        }
    }
}

fn is_sorting_relevant(kind: &EventKind) -> bool {
    matches!(
        kind,
        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) | EventKind::Any
    )
}
