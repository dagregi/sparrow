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

use crate::colors::Colors;

use super::TorrentData;

pub struct TrackersTab<'a> {
    data: &'a TorrentData,
    colors: &'a Colors,
    state: ListState,
    scroll_state: ScrollbarState,
}

impl<'a> TrackersTab<'a> {
    pub fn new(data: &'a TorrentData, colors: &'a Colors) -> Self {
        Self {
            data,
            colors,
            state: ListState::default().with_selected(Some(0)),
            scroll_state: ScrollbarState::new((data.trackers.len()) * 4),
        }
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
