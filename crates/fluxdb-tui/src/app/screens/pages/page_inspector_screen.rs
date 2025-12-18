use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

use fluxdb_core::pager::page::Page;
use fluxdb_core::records::db_record::DbRecord;
use fluxdb_core::records::record::Record;
use fluxdb_core::records::record_type::RecordType;
use fluxdb_core::records::table_column::TableColumn;
use fluxdb_core::records::table_meta::TableMeta;

use crate::app::app_context::AppContext;
use crate::app::screen_action::{Screen, ScreenAction};

pub struct PageInspectorScreen {
    page_id: u64,
    slot_state: ListState,
    page: Option<Page>,
}

impl PageInspectorScreen {
    pub fn new(page_id: u64) -> Self {
        let mut slot_state = ListState::default();
        slot_state.select(Some(0));

        Self {
            page_id,
            slot_state,
            page: None,
        }
    }

    // ───────────────────────── Rendering ─────────────────────────

    fn render_header(&self, f: &mut Frame, area: Rect, page: &Page) {
        let h = &page.header;

        let line = Line::from(vec![
            Span::styled(
                format!("Page {}", h.page_id),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" | "),
            Span::raw(format!("Type: {:?} | ", h.page_type)),
            Span::raw(format!("Slots: {} | ", h.slot_count)),
            Span::raw(format!("FreeStart: {} | ", h.free_start)),
            Span::raw(format!("FreeEnd: {}", h.free_end)),
        ]);

        f.render_widget(
            Paragraph::new(line)
                .block(Block::default().title(" Page Header ").borders(Borders::ALL)),
            area,
        );
    }

    fn render_slots(&self, f: &mut Frame, area: Rect, page: &Page) {
        let items: Vec<ListItem> = page
            .iter_slots()
            .map(|(i, slot)| {
                ListItem::new(format!(
                    "Slot {:02} → offset={} len={}",
                    i, slot.offset, slot.length
                ))
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().title(" Slots ").borders(Borders::ALL))
            .highlight_style(
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");

        let mut state = self.slot_state.clone();
        f.render_stateful_widget(list, area, &mut state);
    }

    fn render_record(&self, f: &mut Frame, area: Rect, page: &Page) {
        let Some(slot_id) = self.slot_state.selected() else {
            f.render_widget(
                Paragraph::new("No slot selected")
                    .block(Block::default().title(" Record ").borders(Borders::ALL)),
                area,
            );
            return;
        };

        let Some(raw) = page.read_record(slot_id as u16) else {
            f.render_widget(
                Paragraph::new("Empty / deleted slot")
                    .block(Block::default().title(" Record ").borders(Borders::ALL)),
                area,
            );
            return;
        };

        let decoded = Self::pretty_print_record(&raw);

        let hex = raw
            .iter()
            .enumerate()
            .map(|(i, b)| {
                if i % 16 == 0 {
                    format!("\n{:04x}: {:02x}", i, b)
                } else {
                    format!(" {:02x}", b)
                }
            })
            .collect::<String>();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(7),
                Constraint::Min(1),
            ])
            .split(area);

        f.render_widget(
            Paragraph::new(decoded)
                .block(Block::default().title(" Decoded ").borders(Borders::ALL)),
            chunks[0],
        );

        f.render_widget(
            Paragraph::new(hex)
                .block(Block::default().title(" Hex ").borders(Borders::ALL)),
            chunks[1],
        );
    }

    fn pretty_print_record(raw: &[u8]) -> String {
        let (record_type, payload) = Record::decode(raw).unwrap();
        
        match record_type {
            RecordType::CatalogTable => {
                match TableMeta::deserialize(payload) {
                    Ok(t) => format!(
                        "CatalogTable\n\
                         ├─ id: {}\n\
                         └─ name: {}",
                        t.table_id,
                        t.name
                    ),
                    Err(e) => format!("❌ CatalogTable decode failed: {e}"),
                }
            }

            RecordType::CatalogColumn => {
                match TableColumn::deserialize(payload) {
                    Ok(c) => format!(
                        "CatalogColumn\n\
                         ├─ table_id: {}\n\
                         └─ name: {}\n",
                        c.table_id,
                        c.name,
                    ),
                    Err(e) => format!("❌ CatalogColumn decode failed: {e}"),
                }
            }

            RecordType::CatalogRoot => {
                "⚠ CatalogRoot (unexpected here)".into()
            }

            other => format!("Unknown record type: {:?}", other),
        }
    }
}

impl Screen for PageInspectorScreen {
    // ───────────────────────── UPDATE ─────────────────────────

    fn update(&mut self, ctx: &AppContext) -> ScreenAction {
        let Some(db) = ctx.db else {
            self.page = None;
            return ScreenAction::None;
        };

        self.page = db.pager.read_page(self.page_id).ok();
        ScreenAction::None
    }

    // ───────────────────────── INPUT ─────────────────────────

    fn handle_event(&mut self, event: Event, _ctx: &AppContext) -> ScreenAction {
        match event {
            Event::Key(KeyEvent {
                           code,
                           kind: KeyEventKind::Press,
                           ..
                       }) => match code {
                KeyCode::Char('q') => ScreenAction::Pop,

                KeyCode::Up => {
                    let i = self.slot_state.selected().unwrap_or(0);
                    self.slot_state.select(Some(i.saturating_sub(1)));
                    ScreenAction::None
                }

                KeyCode::Down => {
                    let max = self
                        .page
                        .as_ref()
                        .map(|p| p.header.slot_count.saturating_sub(1) as usize)
                        .unwrap_or(0);

                    let i = self.slot_state.selected().unwrap_or(0);
                    self.slot_state.select(Some(i.min(max).saturating_add(1).min(max)));
                    ScreenAction::None
                }

                _ => ScreenAction::None,
            },
            _ => ScreenAction::None,
        }
    }

    // ───────────────────────── DRAW ─────────────────────────

    fn draw(&self, f: &mut Frame, _ctx: &AppContext) {
        let Some(page) = &self.page else {
            f.render_widget(
                Paragraph::new("Failed to load page")
                    .block(Block::default().borders(Borders::ALL)),
                f.area(),
            );
            return;
        };

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(1),
            ])
            .split(f.area());

        let body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(32),
                Constraint::Min(1),
            ])
            .split(layout[1]);

        self.render_header(f, layout[0], page);
        self.render_slots(f, body[0], page);
        self.render_record(f, body[1], page);
    }
}
