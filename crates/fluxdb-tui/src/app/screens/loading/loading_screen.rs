use std::path::Path;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::thread;

use crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::Frame;
use fluxdb_core::engine::database::Database;
use crate::app::app_context::AppContext;
use crate::app::screen_action::{Screen, ScreenAction};
use crate::app::screens::loading::load_event::LoadEvent;

pub struct LoadingScreen {
    rx: Receiver<LoadEvent>,
    message: String,
    progress: u16,
}

impl LoadingScreen {
    pub fn new(db_path: String) -> Self {
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            tx.send(LoadEvent::Progress {
                message: "Opening database file…".into(),
                progress: 10,
            }).ok();

            let db = match Database::open(Path::new(&db_path), true) {
                Ok(db) => db,
                Err(e) => {
                    tx.send(LoadEvent::Error(e.to_string())).ok();
                    return;
                }
            };

            tx.send(LoadEvent::Progress {
                message: "Loading catalog…".into(),
                progress: 60,
            }).ok();

            tx.send(LoadEvent::Finished(db)).ok();
        });

        Self {
            rx,
            message: "Starting…".into(),
            progress: 0,
        }
    }
}

impl Screen for LoadingScreen {
    fn handle_event(
        &mut self,
        event: Event,
        _ctx: &AppContext,
    ) -> ScreenAction {
        if let Event::Key(KeyEvent { code: KeyCode::Char('q'), .. }) = event {
            return ScreenAction::Exit;
        }
        ScreenAction::None
    }

    fn update(&mut self, _ctx: &AppContext) -> ScreenAction {
        while let Ok(ev) = self.rx.try_recv() {
            match ev {
                LoadEvent::Progress { message, progress } => {
                    self.message = message;
                    self.progress = progress;
                }

                LoadEvent::Finished(db) => {
                    // 🚨 DO NOT touch ctx or screens here
                    return ScreenAction::SetDatabase(db);
                }

                LoadEvent::Error(err) => {
                    eprintln!("Load error: {err}");
                    return ScreenAction::Exit;
                }
            }
        }

        ScreenAction::None
    }

    fn draw(&self, f: &mut Frame, _ctx: &AppContext) {
        use ratatui::layout::{Constraint, Direction, Layout};
        use ratatui::widgets::{Block, Borders, Gauge, Paragraph};

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .split(f.area());

        let header = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(30),
                Constraint::Length(25),
            ])
            .split(layout[0]);

        f.render_widget(
            Block::default()
                .title(" FluxDB ")
                .borders(Borders::ALL),
            header[0],
        );

        f.render_widget(
            Gauge::default()
                .percent(self.progress)
                .block(Block::default().borders(Borders::ALL)),
            header[1],
        );

        f.render_widget(
            Paragraph::new(self.message.clone())
                .block(Block::default().borders(Borders::ALL)),
            layout[1],
        );
    }
}
