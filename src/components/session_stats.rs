use std::{cell::RefCell, rc::Rc};

use color_eyre::Result;
use futures::executor::block_on;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::Style,
    text::Line,
    widgets::{Block, BorderType, Paragraph},
    Frame,
};
use transmission_rpc::{types::SessionStats, TransClient};

use crate::{action::Action, app, colors::Colors, utils::convert_bytes};

use super::Component;

pub struct SessionStat {
    client: Rc<RefCell<TransClient>>,
    stats: SessionStats,
    colors: Colors,
}

impl Component for SessionStat {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Tick => {
                self.stats = match block_on(get_stats(self.client.clone())) {
                    Ok(stats) => stats,
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

        self.render_stats(frame, rects[1]);
        Ok(())
    }
}

impl SessionStat {
    pub fn new(client: Rc<RefCell<TransClient>>) -> Result<Self> {
        let stats = block_on(get_stats(client.clone()))?;
        Ok(Self {
            client,
            stats,
            colors: Colors::new(),
        })
    }

    fn render_stats(&self, frame: &mut Frame, area: Rect) {
        let stats = &self.stats;
        let stats_text = format!(
            "Down: {}/s Up: {}/s Torrents: {} ",
            convert_bytes(stats.download_speed),
            convert_bytes(stats.upload_speed),
            stats.torrent_count
        );
        let info_footer = Paragraph::new(Line::from(stats_text))
            .style(
                Style::new()
                    .fg(self.colors.row_fg)
                    .bg(self.colors.buffer_bg),
            )
            .right_aligned()
            .block(
                Block::bordered()
                    .border_type(BorderType::Double)
                    .border_style(Style::new().fg(self.colors.footer_border_color)),
            );
        frame.render_widget(info_footer, area);
    }
}

async fn get_stats(client: Rc<RefCell<TransClient>>) -> Result<SessionStats, app::Error> {
    let res = {
        let mut client = client.borrow_mut();
        async move { client.session_stats().await }
    }
    .await;

    match res {
        Ok(stats) => Ok(stats.arguments),
        Err(err) => Err(app::Error::WithMessage(err.to_string())),
    }
}
