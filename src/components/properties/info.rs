use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Text},
    widgets::{Block, Paragraph},
    Frame,
};

use crate::colors::Colors;

use super::TorrentData;

pub struct InfoTab {
    data: TorrentData,
    colors: Colors,
}

impl InfoTab {
    pub fn new(data: &TorrentData) -> Self {
        Self {
            data: data.clone(),
            colors: Colors::new(),
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let rect = Layout::vertical([
            Constraint::Min(5),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(area);

        let activity = vec![
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
            Line::from(format!("State: {}", self.data.status)),
            Line::from(format!("Error: {}", self.data.error)),
        ];
        let details = vec![
            Line::from(format!("Name: {}", self.data.name)),
            Line::from(format!("Size: {}", self.data.total_size)),
            Line::from(format!("Location: {}", self.data.location)),
            Line::from(format!("Hash: {}", self.data.hash)),
            Line::from(format!("Added: {}", self.data.added_date)),
            Line::from(format!("Done: {}", self.data.done_date)),
        ];

        let par_style = Style::default()
            .fg(self.colors.row_fg)
            .bg(self.colors.buffer_bg);
        let border_style = Style::default().fg(self.colors.footer_border_color);

        let activity_par = Paragraph::new(Text::from(activity)).style(par_style).block(
            Block::bordered()
                .border_style(border_style)
                .title("Activity".bold().white()),
        );
        let details_par = Paragraph::new(Text::from(details)).style(par_style).block(
            Block::bordered()
                .border_style(border_style)
                .title("Details".bold().white()),
        );

        frame.render_widget(activity_par, rect[0]);
        frame.render_widget(details_par, rect[1]);
    }
}
