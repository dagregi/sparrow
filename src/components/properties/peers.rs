#![allow(dead_code)]
use ratatui::{layout::Rect, widgets::Paragraph, Frame};

use crate::data;

pub struct Tab {
    data: data::Torrent,
}

impl Tab {
    pub fn new(data: &data::Torrent) -> Self {
        Self { data: data.clone() }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        frame.render_widget(Paragraph::new("Under Construction"), area);
    }
}
