use ratatui::{
    layout::Rect,
    style::Stylize,
    text::{Line, Text},
    widgets::Paragraph,
    Frame,
};

use super::TorrentData;

pub struct InfoTab<'a> {
    data: &'a TorrentData,
}

impl<'a> InfoTab<'a> {
    pub fn new(data: &'a TorrentData) -> Self {
        Self { data }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let activity = vec![
            Line::from("Activity".bold()),
            Line::from(format!(
                "Have: {} of {} ({})",
                self.data.downloaded, self.data.size_done, self.data.percent_done,
            )),
            Line::from(format!(
                "Uploaded: {} (Ratio: {})",
                self.data.uploaded, self.data.ratio
            )),
            Line::from(format!("Downloaded: {}", self.data.downloaded,)),
            Line::from(format!("Remaining Time: {}", self.data.eta)),
            Line::from(format!("State: {}", self.data.is_stalled)),
            Line::from(format!("Error: {}", self.data.error)),
            Line::from("Details".bold()),
            Line::from(format!("Name: {}", self.data.name)),
            Line::from(format!("Size: {}", self.data.total_size)),
            Line::from(format!("Location: {}", self.data.location)),
            Line::from(format!("Hash: {}", self.data.hash)),
            Line::from(format!("Added: {}", self.data.added_date)),
            Line::from(format!("Done: {}", self.data.done_date)),
        ];

        frame.render_widget(Paragraph::new(Text::from(activity)), area);
    }
}
