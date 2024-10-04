use itertools::Itertools;
use ratatui::{
    layout::Rect,
    text::{Line, Text},
    widgets::Paragraph,
    Frame,
};

use super::TorrentData;

pub struct TrackersTab<'a> {
    data: &'a TorrentData,
}

impl<'a> TrackersTab<'a> {
    pub fn new(data: &'a TorrentData) -> Self {
        Self { data }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let lines = self
            .data
            .trackers
            .iter()
            .map(|tracker| {
                Line::from(format!(
                    "{}\t{}\t{}",
                    tracker.host, tracker.next_announce, tracker.is_backup
                ))
            })
            .collect_vec();

        frame.render_widget(Paragraph::new(Text::from(lines)), area);
    }
}
