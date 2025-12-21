use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    prelude::{Modifier, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};
use fluxdb_core::metadata::db_record::DbRecord;
use fluxdb_core::metadata::record::Record;
use fluxdb_core::metadata::record_type::RecordType;
use fluxdb_core::metadata::schema::table_column::TableColumn;
use fluxdb_core::metadata::schema::table_meta::TableMeta;

use fluxdb_core::storage::heap_page_header::HeapPageHeader;
use fluxdb_core::storage::page::Page;
use fluxdb_core::storage::page_header::PageHeader;
use fluxdb_core::storage::page_type::PageType;

use crate::app::{
    app_context::AppContext,
    screen_action::{Screen, ScreenAction},
};

#[derive(Debug, Clone, Copy)]
enum SlotStateKind {
    Live,
    Empty,
    Deleted,
    Corrupt,
}

#[derive(Debug, Clone, Copy)]
enum RecordTab {
    Decoded,
    Payload,
    Hex,
}

pub struct PageInspectorScreen {
    page_id: u64,
    page: Option<Page>,

    slot_state: ListState,
    record_tab: RecordTab,
}

impl PageInspectorScreen {
    pub fn new(page_id: u64) -> Self {
        let mut slot_state = ListState::default();
        slot_state.select(Some(0));

        Self {
            page_id,
            page: None,
            slot_state,
            record_tab: RecordTab::Decoded,
        }
    }

    // ───────────────────────── slot helpers ─────────────────────────

    fn slot_state(page: &Page, slot_id: u16, page_size: usize) -> SlotStateKind {
        let Some(slot) = page.read_slot(slot_id) else {
            return SlotStateKind::Empty;
        };

        if slot.length == 0 {
            return SlotStateKind::Empty;
        }

        if (slot.offset + slot.length) as usize > page_size {
            return SlotStateKind::Corrupt;
        }

        if page.read_record(slot_id).is_none() {
            return SlotStateKind::Deleted;
        }

        SlotStateKind::Live
    }

    fn slot_style(state: SlotStateKind) -> Style {
        match state {
            SlotStateKind::Live => Style::default().fg(Color::Green),
            SlotStateKind::Empty => Style::default().fg(Color::DarkGray),
            SlotStateKind::Deleted => Style::default().fg(Color::Yellow),
            SlotStateKind::Corrupt => {
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
            }
        }
    }

    // ───────────────────────── record decoding ─────────────────────────

    fn decode_payload(
        record_type: RecordType,
        payload: &[u8],
        ctx: &AppContext,
    ) -> String {
        let Some(db) = ctx.db else {
            return "No database loaded".to_string();
        };

        match record_type {
            RecordType::CatalogTable => {
                match TableMeta::deserialize(payload) {
                    Ok(t) => format!(
                        "CatalogTable\n\
                         ────────────\n\
                         id   : {}\n\
                         name : {}",
                        t.table_id,
                        t.name
                    ),
                    Err(e) => format!("❌ Decode failed:\n{e}"),
                }
            }

            RecordType::CatalogColumn => {
                match TableColumn::deserialize(payload) {
                    Ok(c) => {
                        let table_name = db
                            .catalog
                            .tables_by_id
                            .get(&c.table_id)
                            .map(|t| t.name.as_str())
                            .unwrap_or("<unknown>");

                        format!(
                            "CatalogColumn\n\
                             ─────────────\n\
                             table_id : {} ({})\n\
                             name     : {}\n\
                             type     : {:?}",
                            c.table_id,
                            table_name,
                            c.name,
                            c.column_type
                        )
                    }
                    Err(e) => format!("❌ Decode failed:\n{e}"),
                }
            }

            _ => format!("Unsupported record type: {:?}", record_type),
        }
    }

    fn hex_dump(bytes: &[u8]) -> String {
        let mut out = String::new();

        for (row, chunk) in bytes.chunks(16).enumerate() {
            out.push_str(&format!("{:04x}: ", row * 16));
            for b in chunk {
                out.push_str(&format!("{:02x} ", b));
            }
            out.push('\n');
        }

        out
    }

    // ───────────────────────── heap page view ─────────────────────────

    fn render_heap_page(&self, f: &mut Frame, area: Rect, page: &Page, ctx: &AppContext) {
        let db = ctx.db.unwrap();
        let page_size = db.pager.header.page_size as usize;

        let layout = HeapPageHeader::read_from(
            &page.buf[PageHeader::SIZE..PageHeader::SIZE + HeapPageHeader::SIZE],
        );

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(30), // slots
                Constraint::Min(1),     // record
            ])
            .split(area);

        let items: Vec<ListItem> = (0..layout.slot_count)
            .map(|i| {
                let state = Self::slot_state(page, i, page_size);
                ListItem::new(format!("Slot {:02}", i))
                    .style(Self::slot_style(state))
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().title(" Slots ").borders(Borders::ALL))
            .highlight_symbol("▶ ")
            .highlight_style(
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            );

        let mut local = self.slot_state.clone();
        f.render_stateful_widget(list, chunks[0], &mut local);

        self.render_record_panel(f, chunks[1], page, ctx);
    }

    fn render_record_panel(&self, f: &mut Frame, area: Rect, page: &Page, ctx: &AppContext) {
        let Some(slot_id) = self.slot_state.selected() else {
            return;
        };

        let slot_id = slot_id as u16;
        let Some(raw) = page.read_record(slot_id) else {
            f.render_widget(
                Paragraph::new("Empty / Deleted slot")
                    .block(Block::default().borders(Borders::ALL)),
                area,
            );
            return;
        };

        let (rt, payload) = Record::decode(raw).unwrap();

        let header = Line::from(vec![
            Span::styled(
                format!("Slot {}", slot_id),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!(" | {:?}", rt)),
        ]);

        let body = match self.record_tab {
            RecordTab::Decoded => Self::decode_payload(rt, payload, ctx),
            RecordTab::Payload => String::from_utf8_lossy(payload).to_string(),
            RecordTab::Hex => Self::hex_dump(raw),
        };

        let title = match self.record_tab {
            RecordTab::Decoded => " Decoded ",
            RecordTab::Payload => " Payload ",
            RecordTab::Hex => " Hex ",
        };

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(1)])
            .split(area);

        f.render_widget(
            Paragraph::new(header)
                .block(Block::default().borders(Borders::ALL)),
            layout[0],
        );

        f.render_widget(
            Paragraph::new(body)
                .wrap(Wrap { trim: false })
                .block(Block::default().title(title).borders(Borders::ALL)),
            layout[1],
        );
    }

    // ───────────────────────── data page view ─────────────────────────

    fn render_data_page(&self, f: &mut Frame, area: Rect, page: &Page, ctx: &AppContext) {
        let db = ctx.db.unwrap();
        let page_size = db.pager.header.page_size;

        //TODO:
        // let used = page.header.free_start;
        // let free = page.header.free_end.saturating_sub(page.header.free_start);


        let used = 0;
        let free = 0;


        let text = format!(
            "Data Page\n\
             ─────────\n\
             Page id     : {}\n\
             Page size   : {} bytes\n\
             Used bytes  : {}\n\
             Free bytes  : {}\n\
             Utilization : {:.2} %",
            page.header.page_id,
            page_size,
            used,
            free,
            (used as f64 / page_size as f64) * 100.0
        );

        f.render_widget(
            Paragraph::new(text)
                .block(Block::default().title(" Data ").borders(Borders::ALL)),
            area,
        );
    }
}

impl Screen for PageInspectorScreen {
    fn update(&mut self, ctx: &AppContext) -> ScreenAction {
        let Some(db) = ctx.db else {
            self.page = None;
            return ScreenAction::None;
        };

        self.page = db.pager.read_page(self.page_id).ok();

        ScreenAction::None
    }

    fn handle_event(&mut self, event: Event, _ctx: &AppContext) -> ScreenAction {
        if let Event::Key(KeyEvent { code, kind: KeyEventKind::Press, .. }) = event {
            match code {
                KeyCode::Char('q') => return ScreenAction::Pop,

                KeyCode::Up => {
                    let i = self.slot_state.selected().unwrap_or(0);
                    self.slot_state.select(Some(i.saturating_sub(1)));
                }
                KeyCode::Down => {
                    let i = self.slot_state.selected().unwrap_or(0);
                    self.slot_state.select(Some(i + 1));
                }

                KeyCode::Char('1') => self.record_tab = RecordTab::Decoded,
                KeyCode::Char('2') => self.record_tab = RecordTab::Payload,
                KeyCode::Char('3') => self.record_tab = RecordTab::Hex,

                _ => {}
            }
        }

        ScreenAction::None
    }

    fn draw(&self, f: &mut Frame, ctx: &AppContext) {
        let Some(page) = &self.page else {
            f.render_widget(
                Paragraph::new("Failed to load page")
                    .block(Block::default().borders(Borders::ALL)),
                f.size(),
            );
            return;
        };

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(1),
                Constraint::Length(3),
            ])
            .split(f.size());

        let header = Line::from(vec![
            Span::styled(
                format!("Page {}", page.header.page_id),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!(" | {:?}", page.header.page_type)),
        ]);

        f.render_widget(
            Paragraph::new(header)
                .block(Block::default().borders(Borders::ALL)),
            layout[0],
        );

        match page.header.page_type {
            PageType::CatalogPage | PageType::HeapPage => {
                self.render_heap_page(f, layout[1], page, ctx)
            }
            PageType::DataPage => {
                self.render_data_page(f, layout[1], page, ctx)
            }
            _ => {}
        }

        f.render_widget(
            Paragraph::new(
                "[↑↓] Navigate slots  [1/2/3] Decode/Payload/Hex  [q] Back",
            )
                .block(Block::default().borders(Borders::ALL)),
            layout[2],
        );
    }
}
