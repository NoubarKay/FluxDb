use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use crossterm::event::KeyCode::Home;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Borders, Paragraph, List, ListItem, ListState};
use ratatui::Frame;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};

use crate::app::app_context::AppContext;
use crate::app::screen_action::{Screen, ScreenAction};
use crate::app::screens::pages::pages_screen::PagesScreen;

#[derive(Debug, Clone, Copy)]
enum NavItem {
    Pages,
    Catalog,
    Tables,
    Header,
}

impl NavItem {
    fn label(&self) -> &'static str {
        match self {
            NavItem::Pages => "Pages",
            NavItem::Catalog => "Catalog",
            NavItem::Tables => "Tables",
            NavItem::Header => "Header",
        }
    }

    fn create_screen(&self) -> Box<dyn Screen> {
        match self {
            NavItem::Pages => Box::new(PagesScreen::new()),
            NavItem::Catalog => Box::new(HomeScreen::new()),
            NavItem::Tables => Box::new(HomeScreen::new()),
            NavItem::Header => Box::new(HomeScreen::new()),
        }
    }
}

pub struct HomeScreen {
    nav_items: Vec<NavItem>,
    nav_state: ListState,
}

impl HomeScreen {
    pub fn new() -> Self {
        let nav_items = vec![
            NavItem::Pages,
            NavItem::Catalog,
            NavItem::Tables,
            NavItem::Header,
        ];

        let mut nav_state = ListState::default();
        nav_state.select(Some(0));

        Self {
            nav_items,
            nav_state,
        }
    }

    // ───────────────────────────────── Header ─────────────────────────────────

    fn render_header(&self, f: &mut Frame, area: Rect, ctx: &AppContext) {
        let Some(db) = ctx.db else {
            f.render_widget(
                Paragraph::new("No database loaded")
                    .block(Block::default().title(" FluxDB ").borders(Borders::ALL)),
                area,
            );
            return;
        };

        let line = Line::from(vec![
            Span::styled("DB: test.flxdb  ", Style::default().fg(Color::Green)),
            Span::raw("| "),
            Span::raw(format!("PageSize: {}  ", db.pager.header.page_size)),
            Span::raw("| "),
            Span::raw(format!("Ver: {}  ", db.pager.header.db_version)),
            Span::raw("| "),
            Span::raw(format!("Pages: {}  ", db.pager.header.page_count)),
            Span::raw("| "),
            Span::raw(format!(
                "HWM: {}  ",
                db.pager.header.page_count.saturating_sub(1)
            )),
            Span::raw("| "),
            Span::styled("CAT: OK  ", Style::default().fg(Color::Green)),
            Span::raw("| "),
            Span::styled("FLAGS: RO", Style::default().fg(Color::Yellow)),
        ]);

        f.render_widget(
            Paragraph::new(line)
                .block(Block::default().title(" FluxDB ").borders(Borders::ALL)),
            area,
        );
    }

    // ───────────────────────────────── Sidebar ─────────────────────────────────

    fn render_sidebar(&self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .nav_items
            .iter()
            .map(|item| ListItem::new(item.label()))
            .collect();

        let list = List::new(items)
            .block(Block::default().title(" Navigator ").borders(Borders::ALL))
            .highlight_style(Style::default().fg(Color::Yellow))
            .highlight_symbol("▶ ");

        let mut state = self.nav_state.clone();
        f.render_stateful_widget(list, area, &mut state);
    }

    // ───────────────────────────────── Main ─────────────────────────────────

    fn render_main(&self, f: &mut Frame, area: Rect) {
        f.render_widget(
            Paragraph::new("Select a view from the navigator")
                .block(Block::default().title(" Home ").borders(Borders::ALL)),
            area,
        );
    }

    // ───────────────────────────────── Footer ─────────────────────────────────

    fn render_footer(&self, f: &mut Frame, area: Rect) {
        f.render_widget(
            Paragraph::new("[↑↓] Navigate  [Enter] Open  [q] Quit")
                .block(Block::default().borders(Borders::ALL)),
            area,
        );
    }
}

impl Screen for HomeScreen {
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
                KeyCode::Char('q') => ScreenAction::Exit,

                KeyCode::Up => {
                    let i = self.nav_state.selected().unwrap_or(0);
                    self.nav_state.select(Some(i.saturating_sub(1)));
                    ScreenAction::None
                }

                KeyCode::Down => {
                    let i = self.nav_state.selected().unwrap_or(0);
                    let next = (i + 1).min(self.nav_items.len() - 1);
                    self.nav_state.select(Some(next));
                    ScreenAction::None
                }

                KeyCode::Enter => {
                    let i = self.nav_state.selected().unwrap_or(0);
                    ScreenAction::Push(self.nav_items[i].create_screen())
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
                Constraint::Length(3), // header
                Constraint::Min(1),    // body
                Constraint::Length(2), // footer
            ])
            .split(f.area());

        let body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(24), // sidebar
                Constraint::Min(1),     // main
            ])
            .split(layout[1]);

        self.render_header(f, layout[0], ctx);
        self.render_sidebar(f, body[0]);
        self.render_main(f, body[1]);
        self.render_footer(f, layout[2]);
    }
}
