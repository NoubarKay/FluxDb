use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

use fluxdb_core::storage::heap_page_header::HeapPageHeader;
use fluxdb_core::storage::page::Page;
use fluxdb_core::storage::page_header::PageHeader;
use fluxdb_core::storage::page_type::PageType;

use crate::app::app_context::AppContext;
use crate::app::screen_action::{Screen, ScreenAction};
use crate::app::screens::pages::page_inspector_screen::PageInspectorScreen;

#[derive(Copy, Clone)]
enum ViewMode {
    Density,
    Free,
}

pub struct PagesScreen {
    state: ListState,
    mode: ViewMode,
}

impl PagesScreen {
    pub fn new() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        Self {
            state,
            mode: ViewMode::Density,
        }
    }

    // ───────────────────────── PAGE INTENSITY ─────────────────────────

    fn page_intensity(&self, page: &Page) -> u32 {
        match page.header.page_type {
            PageType::CatalogPage | PageType::HeapPage => {
                let layout = HeapPageHeader::read_from(
                    &page.buf[
                        PageHeader::SIZE
                            ..PageHeader::SIZE + HeapPageHeader::SIZE
                        ],
                );
                layout.slot_count as u32
            }

            PageType::DataPage => {
                // Columnar pages: use written bytes as density signal
                //todo
                // page.header.free_start as u32 / 64
                0
            }

            _ => 0,
        }
    }

    // ───────────────────────── STORAGE MINIMAP ─────────────────────────

    fn render_minimap(&self, f: &mut Frame, area: Rect, ctx: &AppContext) {
        let Some(db) = ctx.db else { return };

        let page_count = db.pager.header.page_count as usize;
        let selected = self.state.selected().unwrap_or(0);

        let chars: String = (0..page_count)
            .map(|i| {
                let page = db.pager.read_page(i as u64).ok();

                let intensity = page
                    .as_ref()
                    .map(|p| self.page_intensity(p))
                    .unwrap_or(0);

                let c = match intensity {
                    0 => '░',
                    1..=4 => '▒',
                    5..=16 => '▓',
                    _ => '█',
                };

                if i == selected { '▌' } else { c }
            })
            .collect();

        let title = match self.mode {
            ViewMode::Density => " Storage Minimap (density) ",
            ViewMode::Free => " Storage Minimap (free) ",
        };

        f.render_widget(
            Paragraph::new(chars)
                .block(Block::default().title(title).borders(Borders::ALL))
                .style(Style::default().fg(Color::Cyan)),
            area,
        );
    }

    // ───────────────────────── STATS PANEL ─────────────────────────

    fn render_stats(&self, f: &mut Frame, area: Rect, ctx: &AppContext) {
        let Some(db) = ctx.db else { return };

        let mut heap_pages = 0;
        let mut empty_heap_pages = 0;
        let mut total_slots: u64 = 0;
        let mut max_slots: u16 = 0;
        let mut max_page: u64 = 0;

        let mut data_pages = 0;
        let mut data_bytes_used: u64 = 0;

        for page_id in 0..db.pager.header.page_count {
            let Ok(p) = db.pager.read_page(page_id) else { continue };

            match p.header.page_type {
                PageType::CatalogPage | PageType::HeapPage => {
                    let layout = HeapPageHeader::read_from(
                        &p.buf[
                            PageHeader::SIZE
                                ..PageHeader::SIZE + HeapPageHeader::SIZE
                            ],
                    );

                    heap_pages += 1;
                    total_slots += layout.slot_count as u64;

                    if layout.slot_count == 0 {
                        empty_heap_pages += 1;
                    }

                    if layout.slot_count > max_slots {
                        max_slots = layout.slot_count;
                        max_page = page_id;
                    }
                }

                PageType::DataPage => {
                    data_pages += 1;
                    //todo
                    // data_bytes_used += p.header.free_start as u64;
                }

                _ => {}
            }
        }

        let avg_slots = if heap_pages > 0 {
            total_slots as f64 / heap_pages as f64
        } else {
            0.0
        };

        let text = format!(
            "Storage Stats\n\
             ─────────────\n\
             Total pages      : {}\n\
             Heap pages       : {}\n\
             Data pages       : {}\n\
             Empty heap pages : {}\n\
             Avg slots/page   : {:.2}\n\
             Densest heap     : {} ({} slots)\n\
             Data used        : {} KB",
            db.pager.header.page_count,
            heap_pages,
            data_pages,
            empty_heap_pages,
            avg_slots,
            max_page,
            max_slots,
            data_bytes_used / 1024
        );

        f.render_widget(
            Paragraph::new(text)
                .block(Block::default().title(" Overview ").borders(Borders::ALL)),
            area,
        );
    }

    // ───────────────────────── PAGE LIST ─────────────────────────

    fn render_list(&self, f: &mut Frame, area: Rect, ctx: &AppContext) {
        let Some(db) = ctx.db else { return };

        let items: Vec<ListItem> = (0..db.pager.header.page_count)
            .map(|id| {
                let page = db.pager.read_page(id).ok().unwrap();

                let label = match page.header.page_type {
                    PageType::CatalogPage | PageType::HeapPage => {
                        let layout = HeapPageHeader::read_from(
                            &page.buf[
                                PageHeader::SIZE
                                    ..PageHeader::SIZE + HeapPageHeader::SIZE
                                ],
                        );
                        format!("Page {:03} | HEAP | slots={}", id, layout.slot_count)
                    }

                    PageType::DataPage => {
                        format!(
                            "Page {:03} | DATA | used={} bytes",
                            id,
                            "TODO"
                        )
                    }

                    _ => format!("Page {:03} | {:?}", id, page.header.page_type),
                };

                ListItem::new(label)
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().title(" Pages ").borders(Borders::ALL))
            .highlight_symbol("▶ ")
            .highlight_style(
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            );

        let mut state = self.state.clone();
        f.render_stateful_widget(list, area, &mut state);
    }
}

impl Screen for PagesScreen {
    fn handle_event(&mut self, event: Event, _ctx: &AppContext) -> ScreenAction {
        match event {
            Event::Key(KeyEvent { code, kind: KeyEventKind::Press, .. }) => match code {
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

                KeyCode::Char('m') => {
                    self.mode = match self.mode {
                        ViewMode::Density => ViewMode::Free,
                        ViewMode::Free => ViewMode::Density,
                    };
                    ScreenAction::None
                }

                KeyCode::Enter => {
                    let id = self.state.selected().unwrap_or(0);
                    ScreenAction::Push(Box::new(PageInspectorScreen::new(id as u64)))
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
                Constraint::Length(5),
                Constraint::Min(1),
                Constraint::Length(6),
            ])
            .split(f.area());

        let mid = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(30),
                Constraint::Min(1),
            ])
            .split(layout[1]);

        self.render_minimap(f, layout[0], ctx);
        self.render_list(f, mid[0], ctx);
        self.render_stats(f, mid[1], ctx);

        f.render_widget(
            Paragraph::new(
                "[↑↓] Navigate  [Enter] Inspect  [m] Toggle mode  [q] Back\n\
                 ░ empty ▒ low ▓ medium █ dense ▌ selected",
            )
                .block(Block::default().borders(Borders::ALL)),
            layout[2],
        );
    }
}
