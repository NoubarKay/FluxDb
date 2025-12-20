use std::io;
use std::time::Duration;

use crossterm::event::{self, Event};
use ratatui::DefaultTerminal;
use fluxdb_core::engine::database::Database;
use crate::app::app_context::AppContext;
use crate::app::screen_action::{Screen, ScreenAction};
use crate::app::screens::home::home_screen::HomeScreen;

pub struct App {
    pub exit: bool,
    pub screens: Vec<Box<dyn Screen>>,
    pub database: Option<Database>,
}

impl App {
    fn handle_action(&mut self, action: ScreenAction) {
        match action {
            ScreenAction::None => {}

            ScreenAction::Push(screen) => {
                self.screens.push(screen);
            }

            ScreenAction::Pop => {
                self.screens.pop();
                if self.screens.is_empty() {
                    self.exit = true;
                }
            }

            ScreenAction::Replace(screen) => {
                self.screens.pop();
                self.screens.push(screen);
            }

            ScreenAction::Exit => {
                self.exit = true;
            }

            ScreenAction::SetDatabase(db) => {
                self.database = Some(db);

                // Transition to HomeScreen
                self.screens.clear();
                self.screens.push(Box::new(HomeScreen::new()));
            }
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        loop {
            // ───────────── UPDATE PHASE ─────────────
            let update_action = {
                let ctx = AppContext {
                    db: self.database.as_ref(),
                };

                if let Some(screen) = self.screens.last_mut() {
                    screen.update(&ctx)
                } else {
                    ScreenAction::None
                }
            };

            self.handle_action(update_action);

            // ───────────── DRAW PHASE ─────────────
            terminal.draw(|f| {
                let ctx = AppContext {
                    db: self.database.as_ref(),
                };

                if let Some(screen) = self.screens.last() {
                    screen.draw(f, &ctx);
                }
            })?;

            // ───────────── INPUT PHASE ─────────────
            if event::poll(Duration::from_millis(50))? {
                let ev = event::read()?;

                let input_action = {
                    let ctx = AppContext {
                        db: self.database.as_ref(),
                    };

                    if let Some(screen) = self.screens.last_mut() {
                        screen.handle_event(ev, &ctx)
                    } else {
                        ScreenAction::None
                    }
                };

                self.handle_action(input_action);
            }

            if self.exit {
                break;
            }
        }

        Ok(())
    }
}
