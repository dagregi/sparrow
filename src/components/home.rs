use std::{cell::RefCell, rc::Rc};

use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use futures::executor::block_on;
use itertools::Itertools;
use ratatui::{
    prelude::{Constraint, Frame, Layout, Margin, Modifier, Rect, Style, Stylize, Text},
    widgets::{
        Cell, HighlightSpacing, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table,
        TableState,
    },
};
use tokio::sync::mpsc::UnboundedSender;
use transmission_rpc::{
    types::{self, Id, TorrentAction},
    TransClient,
};
use unicode_width::UnicodeWidthStr;

use super::Component;
use crate::{
    action::Action,
    app::{AppError, Mode},
    colors::Colors,
    config::Config,
    data::{map_torrent_data, TorrentData},
};

const ITEM_HEIGHT: usize = 4;

pub struct Home {
    client: Rc<RefCell<TransClient>>,
    state: TableState,
    items: Vec<TorrentData>,
    longest_item_lens: (u16, u16, u16, u16, u16, u16),
    colors: Colors,
    scroll_state: ScrollbarState,
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
}

impl Home {
    pub fn new(client: Rc<RefCell<TransClient>>, id: Option<i64>) -> Result<Self> {
        let data_vec = block_on(map_torrent_data(&client, None))?;
        let index = match id {
            Some(id) => data_vec
                .iter()
                .enumerate()
                .filter_map(|(i, d)| if d.id == id { Some(i) } else { None })
                .next(),
            None => Some(0),
        };

        Ok(Self {
            client,
            state: TableState::default().with_selected(index),
            longest_item_lens: constraint_len_calculator(&data_vec),
            colors: Colors::new(),
            scroll_state: ScrollbarState::new((data_vec.len()) * ITEM_HEIGHT),
            items: data_vec,
            command_tx: None,
            config: Config::default(),
        })
    }

    async fn toggle_state(&mut self) -> types::Result<()> {
        let id = self
            .items
            .get(self.state.selected().ok_or(AppError::NoRowSelected)?)
            .ok_or(AppError::OutOfBound)?
            .id;
        let state = self
            .items
            .get(self.state.selected().ok_or(AppError::NoRowSelected)?)
            .ok_or(AppError::OutOfBound)?
            .is_stalled;
        let mut client = self.client.borrow_mut();
        async move {
            if state {
                client
                    .torrent_action(TorrentAction::Start, vec![Id::Id(id)])
                    .await
            } else {
                client
                    .torrent_action(TorrentAction::Stop, vec![Id::Id(id)])
                    .await
            }
        }
        .await?;
        Ok(())
    }

    async fn start_all(&mut self) -> types::Result<()> {
        let mut client = self.client.borrow_mut();
        let ids = self.items.iter().map(|t| Id::Id(t.id)).collect_vec();
        async move { client.torrent_action(TorrentAction::Start, ids).await }.await?;
        Ok(())
    }

    async fn stop_all(&mut self) -> types::Result<()> {
        let mut client = self.client.borrow_mut();
        let ids = self.items.iter().map(|t| Id::Id(t.id)).collect_vec();
        async move { client.torrent_action(TorrentAction::Stop, ids).await }.await?;
        Ok(())
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i * ITEM_HEIGHT);
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i * ITEM_HEIGHT);
    }

    fn top(&mut self) {
        self.state.select_first();
        self.scroll_state.first();
    }

    fn bottom(&mut self) {
        self.state.select_last();
        self.scroll_state.last();
    }
}

impl Home {
    fn render_table(&mut self, frame: &mut Frame, area: Rect) {
        let header_style = Style::default()
            .fg(self.colors.header_fg)
            .bg(self.colors.header_bg);
        let selected_style = Style::default()
            .add_modifier(Modifier::REVERSED)
            .fg(self.colors.selected_style_fg);

        let header = ["NAME", "DONE", "ETA", "DOWN", "UP", "RATIO"]
            .into_iter()
            .map(Cell::from)
            .collect::<Row>()
            .style(header_style)
            .height(1);
        let rows = self.items.iter().enumerate().map(|(i, data)| {
            let color = match i % 2 {
                0 => self.colors.normal_row_color,
                _ => self.colors.alt_row_color,
            };
            let item = data.ref_array();
            item.into_iter()
                .map(|content| Cell::from(Text::from(format!("\n{content}\n"))))
                .collect::<Row>()
                .style(Style::new().fg(self.colors.row_fg).bg(color))
                .height(4)
        });
        let bar = " █ ";
        let t = Table::new(
            rows,
            [
                Constraint::Length(self.longest_item_lens.0 + 1),
                Constraint::Min(self.longest_item_lens.1 + 1),
                Constraint::Min(self.longest_item_lens.2 + 1),
                Constraint::Min(self.longest_item_lens.3 + 1),
                Constraint::Min(self.longest_item_lens.4 + 1),
                Constraint::Min(self.longest_item_lens.5 + 1),
            ],
        )
        .header(header)
        .highlight_style(selected_style)
        .highlight_symbol(Text::from(vec![
            "".into(),
            bar.into(),
            bar.into(),
            "".into(),
        ]))
        .bg(self.colors.buffer_bg)
        .highlight_spacing(HighlightSpacing::Always);
        frame.render_stateful_widget(t, area, &mut self.state);
    }

    fn render_scrollbar(&mut self, frame: &mut Frame, area: Rect) {
        frame.render_stateful_widget(
            Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None),
            area.inner(Margin {
                vertical: 1,
                horizontal: 1,
            }),
            &mut self.scroll_state,
        );
    }
}
impl Component for Home {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.config = config;
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<Option<Action>> {
        match key_event.code {
            KeyCode::Char('q') => {
                return Ok(Some(Action::Quit));
            }
            KeyCode::Char('l') | KeyCode::Enter => {
                let id = self
                    .items
                    .get(self.state.selected().ok_or(AppError::NoRowSelected)?)
                    .ok_or(AppError::OutOfBound)?
                    .id;
                return Ok(Some(Action::Mode(Mode::Properties, id)));
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.next();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.previous();
            }
            KeyCode::Char('g') => {
                self.top();
            }
            KeyCode::Char('G') => {
                self.bottom();
            }
            KeyCode::Char('p') => {
                match block_on(self.toggle_state()) {
                    Ok(()) => {}
                    Err(err) => return Ok(Some(Action::Error(err.to_string()))),
                };
            }
            KeyCode::Char('s') => {
                match block_on(self.start_all()) {
                    Ok(()) => {}
                    Err(err) => return Ok(Some(Action::Error(err.to_string()))),
                };
            }
            KeyCode::Char('S') => {
                match block_on(self.stop_all()) {
                    Ok(()) => {}
                    Err(err) => return Ok(Some(Action::Error(err.to_string()))),
                };
            }
            // Other handlers you could add here.
            _ => {}
        }
        Ok(None)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Tick => {
                self.items = match block_on(map_torrent_data(&self.client, None)) {
                    Ok(items) => items,
                    Err(err) => return Ok(Some(Action::Error(err.to_string()))),
                };
            }
            Action::Render => {}
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let vertical = &Layout::vertical([Constraint::Min(5), Constraint::Length(3)]);
        let rects = vertical.split(area);

        self.render_table(frame, rects[0]);
        self.render_scrollbar(frame, rects[0]);
        Ok(())
    }
}

fn constraint_len_calculator(items: &[TorrentData]) -> (u16, u16, u16, u16, u16, u16) {
    let name_len = items
        .iter()
        .map(TorrentData::formatted_name)
        .map(UnicodeWidthStr::width)
        .min()
        .unwrap_or(0);
    let done_len = items
        .iter()
        .map(TorrentData::percent_done)
        .flat_map(str::lines)
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or(0);
    let eta_len = items
        .iter()
        .map(TorrentData::eta)
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or(0);
    let up_len = items
        .iter()
        .map(TorrentData::upload_speed)
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or(0);
    let down_len = items
        .iter()
        .map(TorrentData::download_speed)
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or(0);
    let ratio_len = items
        .iter()
        .map(TorrentData::ratio)
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or(0);

    #[allow(clippy::cast_possible_truncation)]
    (
        name_len as u16,
        done_len as u16,
        eta_len as u16,
        down_len as u16,
        up_len as u16,
        ratio_len as u16,
    )
}
