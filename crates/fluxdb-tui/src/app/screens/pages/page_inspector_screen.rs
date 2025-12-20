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
use fluxdb_core::storage::page::Page;
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
enum ByteRegion {
    Header,
    SlotDir,
    Data,
    Free,
    Frag,
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

    // heatmap scroll
    heatmap_row_offset: usize,
    heatmap_col_offset: usize,
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
            heatmap_row_offset: 0,
            heatmap_col_offset: 0,
        }
    }

    // ───────────────────────── helpers ─────────────────────────

    fn page_bytes(page: &Page) -> &[u8] {
        &page.buf
    }

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

    fn region_style(region: ByteRegion) -> Style {
        match region {
            ByteRegion::Header => Style::default().fg(Color::Blue),
            ByteRegion::SlotDir => Style::default().fg(Color::Magenta),
            ByteRegion::Data => Style::default().fg(Color::Cyan),
            ByteRegion::Free => Style::default().fg(Color::DarkGray),
            ByteRegion::Frag => Style::default().fg(Color::Yellow),
        }
    }

    fn classify_region(page: &Page, idx: usize, page_size: usize) -> ByteRegion {
        const HEADER_BYTES: usize = 32;
        let slot_bytes = page.header.slot_count as usize * 4;
        let slot_start = page_size.saturating_sub(slot_bytes);

        if idx < HEADER_BYTES {
            ByteRegion::Header
        } else if idx >= slot_start {
            ByteRegion::SlotDir
        } else if idx < page.header.free_start as usize {
            ByteRegion::Data
        } else if idx < page.header.free_end as usize {
            ByteRegion::Free
        } else {
            ByteRegion::Frag
        }
    }

    fn selected_record_range(&self, page: &Page) -> Option<(usize, usize)> {
        let slot_id = self.slot_state.selected()? as u16;
        let slot = page.read_slot(slot_id)?;
        if slot.length == 0 {
            None
        } else {
            Some((
                slot.offset as usize,
                (slot.offset + slot.length) as usize,
            ))
        }
    }

    // ───────────────────────── header ─────────────────────────

    fn render_header(&self, f: &mut Frame, area: Rect, page: &Page, ctx: &AppContext) {
        let db = ctx.db.unwrap();
        let h = &page.header;

        let line = Line::from(vec![
            Span::styled(
                format!("Page {}", h.page_id),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!(
                " | {:?} | slots={} | free={}..{} | size={}",
                h.page_type,
                h.slot_count,
                h.free_start,
                h.free_end,
                db.pager.header.page_size
            )),
        ]);

        f.render_widget(
            Paragraph::new(line)
                .block(Block::default().title(" Page ").borders(Borders::ALL)),
            area,
        );
    }

    // ───────────────────────── heatmap (SIDE) ─────────────────────────

    fn render_heatmap(&self, f: &mut Frame, area: Rect, page: &Page, ctx: &AppContext) {
        let db = ctx.db.unwrap();
        let page_size = db.pager.header.page_size as usize;

        let bytes_per_row = 32;
        let visible_rows = area.height.saturating_sub(2) as usize;

        let start_row = self.heatmap_row_offset;
        let end_row = ((page_size + bytes_per_row - 1) / bytes_per_row)
            .min(start_row + visible_rows);

        let selected = self.selected_record_range(page);

        let mut lines = Vec::new();

        for row in start_row..end_row {
            let row_start = row * bytes_per_row + self.heatmap_col_offset;
            let row_end = (row_start + bytes_per_row).min(page_size);

            let mut spans = Vec::new();

            for i in row_start..row_end {
                let mut style =
                    Self::region_style(Self::classify_region(page, i, page_size));

                if let Some((rs, re)) = selected {
                    if i >= rs && i < re {
                        style =
                            style.add_modifier(Modifier::REVERSED | Modifier::BOLD);
                    }
                }

                spans.push(Span::styled("█", style));
            }

            lines.push(Line::from(spans));
        }

        f.render_widget(
            Paragraph::new(lines)
                .wrap(Wrap { trim: false })
                .block(Block::default().title(" Heatmap ").borders(Borders::ALL)),
            area,
        );
    }

    // ───────────────────────── record panel ─────────────────────────

    fn render_record(&self, f: &mut Frame, area: Rect, page: &Page, ctx: &AppContext) {
        let slot_id = self.slot_state.selected().unwrap() as u16;
        let slot = page.read_slot(slot_id).unwrap();
        let raw = page.read_record(slot_id).unwrap();
        let (rt, payload) = Record::decode(raw).unwrap();

        let header = Line::from(vec![
            Span::styled(
                format!("Slot {}", slot_id),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!(
                " | {:?} | off={} len={} payload={}",
                rt,
                slot.offset,
                slot.length,
                payload.len()
            )),
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

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(1)])
            .split(area);

        f.render_widget(
            Paragraph::new(header)
                .block(Block::default().borders(Borders::ALL)),
            chunks[0],
        );

        f.render_widget(
            Paragraph::new(body)
                .wrap(Wrap { trim: false })
                .block(Block::default().title(title).borders(Borders::ALL)),
            chunks[1],
        );
    }

    // ───────────────────────── footer ─────────────────────────

    fn render_footer(&self, f: &mut Frame, area: Rect) {
        let help = Line::from(vec![
            Span::styled("↑↓", Style::default().fg(Color::Yellow)),
            Span::raw(" slots  "),
            Span::styled("1/2/3", Style::default().fg(Color::Cyan)),
            Span::raw(" decoded/payload/hex  "),
            Span::styled("PgUp/PgDn", Style::default().fg(Color::Green)),
            Span::raw(" heatmap scroll  "),
            Span::styled("← →", Style::default().fg(Color::Green)),
            Span::raw(" pan  "),
            Span::styled("q", Style::default().fg(Color::Red)),
            Span::raw(" back"),
        ]);

        f.render_widget(
            Paragraph::new(help).block(Block::default().borders(Borders::ALL)),
            area,
        );
    }

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
                    Err(e) => format!("❌ CatalogTable decode failed:\n{e}"),
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
                    Err(e) => format!("❌ CatalogColumn decode failed:\n{e}"),
                }
            }

            _ => format!("Unsupported record type: {:?}", record_type),
        }
    }

    fn hex_dump(bytes: &[u8]) -> String {
        let mut out = String::new();

        for (row, chunk) in bytes.chunks(16).enumerate() {
            out.push_str(&format!("{:04x}: ", row * 16));

            for (i, b) in chunk.iter().enumerate() {
                if i == 8 {
                    out.push(' ');
                }
                out.push_str(&format!("{:02x} ", b));
            }

            out.push('\n');
        }

        out
    }
}

impl Screen for PageInspectorScreen {
    fn update(&mut self, ctx: &AppContext) -> ScreenAction {
        let Some(db) = ctx.db else {
            self.page = None;
            return ScreenAction::None;
        };

        self.page = db.pager.read_page(self.page_id).ok();

        if let Some(page) = &self.page {
            let max = page.header.slot_count.saturating_sub(1) as usize;
            let sel = self.slot_state.selected().unwrap_or(0);
            self.slot_state.select(Some(sel.min(max)));
        }

        ScreenAction::None
    }

    fn handle_event(&mut self, event: Event, _ctx: &AppContext) -> ScreenAction {
        if let Event::Key(KeyEvent {
                              code,
                              kind: KeyEventKind::Press,
                              ..
                          }) = event
        {
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

                KeyCode::PageUp => {
                    self.heatmap_row_offset = self.heatmap_row_offset.saturating_sub(4)
                }
                KeyCode::PageDown => self.heatmap_row_offset += 4,

                KeyCode::Left => {
                    self.heatmap_col_offset = self.heatmap_col_offset.saturating_sub(16)
                }
                KeyCode::Right => self.heatmap_col_offset += 16,

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

        self.render_header(f, layout[0], page, ctx);

        let body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(34), // slots
                Constraint::Min(1),     // record
                Constraint::Length(34), // heatmap
            ])
            .split(layout[1]);

        let page_size = ctx.db.unwrap().pager.header.page_size as usize;

        let items: Vec<ListItem> = (0..page.header.slot_count)
            .map(|i| {
                let slot = page.read_slot(i).unwrap();
                let state = Self::slot_state(page, i, page_size);
                ListItem::new(format!(
                    "Slot {:02} | {:?} | off={} len={}",
                    i, state, slot.offset, slot.length
                ))
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
        f.render_stateful_widget(list, body[0], &mut local);

        self.render_record(f, body[1], page, ctx);
        self.render_heatmap(f, body[2], page, ctx);

        self.render_footer(f, layout[2]);
    }
}
