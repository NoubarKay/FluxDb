use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use ratatui::style::{Color, Style};
use ratatui::Frame;

use crate::app::app_context::AppContext;
use crate::app::screen_action::{Screen, ScreenAction};
use crate::app::screens::pages::page_inspector_screen::PageInspectorScreen;

pub struct PagesScreen {
    state: ListState,
}

impl PagesScreen {
    pub fn new() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));

        Self { state }
    }

    fn render(&self, f: &mut Frame, area: Rect, ctx: &AppContext) {
        let Some(db) = ctx.db else {
            f.render_widget(
                Paragraph::new("No database loaded")
                    .block(Block::default().title(" Pages ").borders(Borders::ALL)),
                area,
            );
            return;
        };

        let page_count = db.pager.header.page_count;

        let items: Vec<ListItem> = (0..page_count)
            .map(|id| ListItem::new(format!("Page {}", id)))
            .collect();

        let list = List::new(items)
            .block(Block::default().title(" Pages ").borders(Borders::ALL))
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(ratatui::style::Modifier::BOLD),
            )
            .highlight_symbol("▶ ");

        let mut state = self.state.clone();
        f.render_stateful_widget(list, area, &mut state);
    }
}

impl Screen for PagesScreen {
    fn handle_event(
        &mut self,
        event: Event,
        _ctx: &AppContext,
    ) -> ScreenAction {
        match event {
            Event::Key(KeyEvent {
                           code,
                           kind: KeyEventKind::Press,
                           ..
                       }) => match code {
                KeyCode::Char('q') => ScreenAction::Pop,

                KeyCode::Up => {
                    let i = self.state.selected().unwrap_or(0);
                    self.state.select(Some(i.saturating_sub(1)));
                    ScreenAction::None
                }

                KeyCode::Down => {
                    let i = self.state.selected().unwrap_or(0);
                    self.state.select(Some(i + 1));
                    ScreenAction::None
                }

                KeyCode::Home => {
                    self.state.select(Some(0));
                    ScreenAction::None
                }

                KeyCode::End => {
                    // Upper bound will clamp in render
                    self.state.select(Some(usize::MAX));
                    ScreenAction::None
                }

                KeyCode::Enter => {
                    let page_id = self.state.selected().unwrap_or(0);
                    ScreenAction::Push(Box::new(PageInspectorScreen::new(page_id as u64)))
                }

                _ => ScreenAction::None,
            },

            _ => ScreenAction::None,
        }
    }

    fn draw(&self, f: &mut Frame, ctx: &AppContext) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),
                Constraint::Length(2),
            ])
            .split(f.area());

        self.render(f, layout[0], ctx);

        f.render_widget(
            Paragraph::new("[↑↓] Navigate  [Home/End] Jump  [Enter] Inspect  [q] Back")
                .block(Block::default().borders(Borders::ALL)),
            layout[1],
        );
    }
}
