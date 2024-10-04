use itertools::Itertools;
use ratatui::{
    layout::Rect,
    text::{Line, Text},
    widgets::Paragraph,
    Frame,
};

use super::TorrentData;

pub struct FilesTab<'a> {
    data: &'a TorrentData,
}

impl<'a> FilesTab<'a> {
    pub fn new(data: &'a TorrentData) -> Self {
        Self { data }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let files = self
            .data
            .files
            .iter()
            .map(|file| {
                Line::from(format!(
                    "{}\t{}/{}\t{}\t{}",
                    file.name, file.downloaded, file.total_size, file.wanted, file.priority
                ))
            })
            .collect_vec();

        frame.render_widget(Paragraph::new(Text::from(files)), area);
    }
}
