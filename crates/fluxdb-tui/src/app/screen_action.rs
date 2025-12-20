use ratatui::Frame;
use fluxdb_core::engine::database::Database;
use crate::app::app_context::AppContext;

pub enum ScreenAction {
    None,
    Push(Box<dyn Screen>),
    Pop,
    Replace(Box<dyn Screen>),
    SetDatabase(Database),
    Exit,
}

pub trait Screen {
    fn handle_event(
        &mut self,
        event: crossterm::event::Event,
        ctx: &AppContext,
    ) -> ScreenAction;


    fn update(&mut self, ctx: &AppContext) -> ScreenAction {
        ScreenAction::None
    }

    fn draw(&self, f: &mut Frame, ctx: &AppContext);
}