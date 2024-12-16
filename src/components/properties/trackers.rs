use itertools::Itertools;
use ratatui::{
    layout::{Constraint, Layout, Margin, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Text},
    widgets::{
        Block, HighlightSpacing, List, ListState, Scrollbar, ScrollbarOrientation, ScrollbarState,
    },
    Frame,
};

use crate::{colors::Colors, data};

const ITEM_HEIGHT: usize = 4;

pub struct Tab {
    data: data::Torrent,
    colors: Colors,
    state: ListState,
    scroll_state: ScrollbarState,
}

impl Tab {
    pub fn new(data: &data::Torrent) -> Self {
        Self {
            data: data.clone(),
            colors: Colors::new(),
            state: ListState::default().with_selected(Some(0)),
            scroll_state: ScrollbarState::new((data.trackers.len()) * ITEM_HEIGHT),
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.data.trackers.len() - 1 {
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

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.data.trackers.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i * ITEM_HEIGHT);
    }

    pub fn top(&mut self) {
        self.state.select_first();
        self.scroll_state.first();
    }

    pub fn bottom(&mut self) {
        self.state.select_last();
        self.scroll_state.last();
    }

    pub fn scroll_up(&mut self, amount: usize) {
        self.state
            .scroll_up_by(u16::try_from(amount).expect("failed to parse"));
        self.scroll_state = self
            .scroll_state
            .position(self.state.selected().unwrap_or(0) * amount);
    }

    pub fn scroll_down(&mut self, amount: usize) {
        self.state
            .scroll_down_by(u16::try_from(amount).expect("failed to parse"));
        self.scroll_state = self
            .scroll_state
            .position(self.state.selected().unwrap_or(0) * amount);
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let rects = Layout::vertical([Constraint::Min(5), Constraint::Length(3)]).split(area);
        let list_style = Style::default()
            .fg(self.colors.row_fg)
            .bg(self.colors.buffer_bg);
        let border_style = Style::default().fg(self.colors.footer_border_color);
        let selected_style = Style::default()
            .add_modifier(Modifier::REVERSED)
            .fg(self.colors.selected_style_fg);

        let items = self
            .data
            .trackers
            .iter()
            .enumerate()
            .map(|(i, tracker)| {
                let host = Line::raw(tracker.host.to_string());
                let update = Line::raw(tracker.next_announce.to_string());

                let color = match i % 2 {
                    0 => self.colors.normal_row_color,
                    _ => self.colors.alt_row_color,
                };

                if tracker.is_backup {
                    Text::from(vec![host.gray(), Line::raw("")])
                } else {
                    Text::from(vec![Line::raw(""), host.bold(), update, Line::raw("")])
                }
                .style(Style::new().fg(self.colors.row_fg).bg(color))
            })
            .collect_vec();
        let list = List::new(items)
            .highlight_style(selected_style)
            .highlight_spacing(HighlightSpacing::Always)
            .style(list_style)
            .block(Block::bordered().border_style(border_style));

        frame.render_stateful_widget(list, rects[0], &mut self.state);
        frame.render_stateful_widget(
            Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None),
            rects[0].inner(Margin {
                vertical: 1,
                horizontal: 1,
            }),
            &mut self.scroll_state,
        );
    }
}
