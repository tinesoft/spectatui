use std::time::Duration;

use crossterm::event::{self, Event as CtEvent, KeyEvent, MouseEvent};
use tokio::sync::mpsc;

use spectatui_core::speckit::watch::FsEvent;
use spectatui_core::speckit::{ExtensionInfo, IntegrationInfo, PresetInfo, WorkflowInfo};
use spectatui_core::tmux::TmuxSession;

#[allow(dead_code)]
pub enum AppEvent {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Tick,
    FsChanged(FsEvent),
    TmuxChanged {
        sessions: Vec<String>,
        session: Option<TmuxSession>,
    },
    Resize(u16, u16),
    CatalogIndexed {
        integrations: Vec<IntegrationInfo>,
        extensions: Vec<ExtensionInfo>,
        presets: Vec<PresetInfo>,
        workflows: Vec<WorkflowInfo>,
    },
}

pub struct EventStream {
    rx: mpsc::UnboundedReceiver<AppEvent>,
}

impl EventStream {
    pub fn new(tick_rate: Duration) -> (Self, mpsc::UnboundedSender<AppEvent>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let tx_clone = tx.clone();

        tokio::spawn(async move {
            loop {
                if event::poll(tick_rate).unwrap_or(false) {
                    match event::read() {
                        Ok(CtEvent::Key(key)) => {
                            if tx_clone.send(AppEvent::Key(key)).is_err() {
                                return;
                            }
                        }
                        Ok(CtEvent::Mouse(mouse)) => {
                            if tx_clone.send(AppEvent::Mouse(mouse)).is_err() {
                                return;
                            }
                        }
                        Ok(CtEvent::Resize(w, h)) => {
                            if tx_clone.send(AppEvent::Resize(w, h)).is_err() {
                                return;
                            }
                        }
                        _ => {}
                    }
                } else if tx_clone.send(AppEvent::Tick).is_err() {
                    return;
                }
            }
        });

        (Self { rx }, tx)
    }

    pub async fn next(&mut self) -> Option<AppEvent> {
        self.rx.recv().await
    }
}
