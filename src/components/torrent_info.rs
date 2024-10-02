use std::{cell::RefCell, rc::Rc};

use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use futures::executor::block_on;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{palette::tailwind, Color, Stylize},
    text::{Line, Text},
    widgets::{Paragraph, Tabs},
    Frame,
};
use strum::{Display, EnumIter, FromRepr, IntoEnumIterator};
use transmission_rpc::{
    types::{self, Id, Torrent},
    TransClient,
};

use crate::{
    action::Action,
    app::Mode,
    colors::Colors,
    utils::{convert_bytes, convert_eta, convert_percentage, convert_status, handle_ratio},
};

use super::Component;

pub struct TorrentInfo {
    client: Rc<RefCell<TransClient>>,
    torrent: Torrent,
    selected_tab: SelectedTab,
    color: Colors,
}

#[derive(Default, Clone, Copy, Display, FromRepr, EnumIter)]
enum SelectedTab {
    #[default]
    #[strum(to_string = "Info")]
    Info,
    #[strum(to_string = "Peers")]
    Peers,
    #[strum(to_string = "Tracker")]
    Tracker,
    #[strum(to_string = "Files")]
    Files,
}

impl Component for TorrentInfo {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        self.render_tabs(frame, area);
        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Tick => {
                self.torrent = match block_on(get_torrent(1, &self.client.clone())) {
                    Ok(t) => t,
                    Err(err) => return Ok(Some(Action::Error(err.to_string()))),
                };
            }
            Action::Render => {}
            _ => {}
        }
        Ok(None)
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        match key.code {
            KeyCode::Char('q') => {
                return Ok(Some(Action::Quit));
            }
            KeyCode::Esc => {
                return Ok(Some(Action::Mode(Mode::Home)));
            }
            KeyCode::Char('l') | KeyCode::Right => self.next_tab(),
            KeyCode::Char('h') | KeyCode::Left => self.previous_tab(),
            _ => {}
        }
        Ok(None)
    }
}

impl TorrentInfo {
    pub fn new(client: Rc<RefCell<TransClient>>) -> Self {
        let torrent = block_on(get_torrent(1, &client)).unwrap();
        Self {
            client,
            torrent,
            selected_tab: SelectedTab::Info,
            color: Colors::new(),
        }
    }

    pub fn next_tab(&mut self) {
        self.selected_tab = self.selected_tab.next();
    }

    pub fn previous_tab(&mut self) {
        self.selected_tab = self.selected_tab.previous();
    }

    fn render_tabs(&self, frame: &mut Frame, area: Rect) {
        let titles = SelectedTab::iter().map(SelectedTab::title);
        let highlight_style = (Color::default(), self.color.tab_selected);
        let selected_tab_index = self.selected_tab as usize;
        let tabs = Tabs::new(titles)
            .highlight_style(highlight_style)
            .select(selected_tab_index)
            .padding("", "")
            .divider(" ");

        let rects = Layout::vertical([Constraint::Length(2), Constraint::Min(5)]).split(area);

        frame.render_widget(tabs, rects[0]);
        match self.selected_tab {
            SelectedTab::Info => self
                .selected_tab
                .render_info(frame, rects[1], &self.torrent),
            SelectedTab::Peers => self.selected_tab.render_peers(frame, rects[1]),
            SelectedTab::Tracker => self.selected_tab.render_tracker(frame, rects[1]),
            SelectedTab::Files => self.selected_tab.render_files(frame, rects[1]),
        }
    }
}

async fn get_torrent(id: i64, client: &Rc<RefCell<TransClient>>) -> types::Result<Torrent> {
    let res = {
        let mut client = client.borrow_mut();
        async move { client.torrent_get(None, Some(vec![Id::Id(id)])).await }
    }
    .await;

    let torrent = match res {
        Ok(t) => t.arguments.torrents[0].clone(),
        Err(err) => return Err(err),
    };

    Ok(torrent)
}

impl SelectedTab {
    /// Get the previous tab, if there is no previous tab return the current tab.
    fn previous(self) -> Self {
        let current_index: usize = self as usize;
        let previous_index = current_index.saturating_sub(1);
        Self::from_repr(previous_index).unwrap_or(self)
    }

    /// Get the next tab, if there is no next tab return the current tab.
    fn next(self) -> Self {
        let current_index = self as usize;
        let next_index = current_index.saturating_add(1);
        Self::from_repr(next_index).unwrap_or(self)
    }
}

impl SelectedTab {
    fn render_info(self, frame: &mut Frame, area: Rect, torrent: &Torrent) {
        let activity = vec![
            Line::from("Activity".bold()),
            Line::from(format!(
                "Have: {} of {} ({})",
                convert_bytes(torrent.size_when_done.unwrap() - torrent.left_until_done.unwrap()),
                convert_bytes(torrent.size_when_done.unwrap()),
                convert_percentage(torrent.percent_done.unwrap()),
            )),
            Line::from(format!(
                "Uploaded: {} (Ratio: {})",
                convert_bytes(torrent.uploaded_ever.unwrap()),
                handle_ratio(torrent.upload_ratio.unwrap()),
            )),
            Line::from(format!(
                "Downloaded: {}",
                convert_bytes(torrent.size_when_done.unwrap() - torrent.left_until_done.unwrap()),
            )),
            Line::from(format!(
                "Remaining Time: {}",
                convert_eta(torrent.eta.unwrap())
            )),
            Line::from(format!(
                "State: {}",
                convert_status(torrent.status.unwrap())
            )),
            Line::from("Details".bold()),
            Line::from(format!(
                "Size: {}",
                convert_bytes(torrent.total_size.unwrap()),
            )),
            Line::from(format!(
                "Location: {}",
                torrent.download_dir.clone().unwrap()
            )),
            Line::from(format!("Hash: {}", torrent.hash_string.clone().unwrap())),
        ];
        let act = Paragraph::new(Text::from(activity));
        frame.render_widget(act, area);
    }
    fn render_peers(self, frame: &mut Frame, area: Rect) {
        frame.render_widget(Paragraph::new("Peers"), area);
    }
    fn render_tracker(self, frame: &mut Frame, area: Rect) {
        frame.render_widget(Paragraph::new("Trackers"), area);
    }
    fn render_files(self, frame: &mut Frame, area: Rect) {
        frame.render_widget(Paragraph::new("Files"), area);
    }

    /// Return tab's name as a styled `Line`
    fn title(self) -> Line<'static> {
        format!("  {self}  ")
            .fg(tailwind::SLATE.c200)
            .bg(self.colors().tab_title_bg)
            .into()
    }

    const fn colors(self) -> Colors {
        Colors::new()
    }
}
