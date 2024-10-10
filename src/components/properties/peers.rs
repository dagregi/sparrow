#![allow(dead_code)]
use ratatui::{layout::Rect, widgets::Paragraph, Frame};

use super::TorrentData;

pub struct PeersTab<'a> {
    data: &'a TorrentData,
}

impl<'a> PeersTab<'a> {
    pub fn new(data: &'a TorrentData) -> Self {
        Self { data }
    }

    pub fn render(self, frame: &mut Frame, area: Rect) {
        frame.render_widget(Paragraph::new("Under Construction"), area);
    }
}
