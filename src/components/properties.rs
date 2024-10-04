use std::{cell::RefCell, rc::Rc};

use chrono::{DateTime, Utc};
use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use files::FilesTab;
use futures::executor::block_on;
use info::InfoTab;
use itertools::Itertools;
use peers::PeersTab;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{palette::tailwind, Modifier, Style, Stylize},
    text::Line,
    widgets::Tabs,
    Frame,
};
use strum::{Display, EnumIter, FromRepr, IntoEnumIterator};
use trackers::TrackersTab;
use transmission_rpc::{types::Id, TransClient};

use crate::{
    action::Action,
    app::{AppError, Mode},
    colors::Colors,
    utils::{convert_bytes, convert_eta, convert_percentage, convert_priority, handle_ratio},
};

use super::Component;

pub mod files;
pub mod info;
pub mod peers;
pub mod trackers;

pub struct Properties {
    client: Rc<RefCell<TransClient>>,
    data: TorrentData,
    selected_tab: SelectedTab,
    colors: Colors,
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Display, FromRepr, EnumIter)]
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

impl Component for Properties {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        self.render_tabs(frame, area);
        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Tick => {
                self.data = match block_on(map_torrent_data(&self.client, 1)) {
                    Ok(d) => d,
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
            KeyCode::Char('l') | KeyCode::Right => {
                self.next_tab();
            }
            KeyCode::Char('h') | KeyCode::Left => {
                self.previous_tab();
                if SelectedTab::Tracker == self.selected_tab {
                    return Ok(Some(Action::ClearScreen));
                }
            }
            _ => {}
        }
        Ok(None)
    }
}

impl Properties {
    pub fn new(client: Rc<RefCell<TransClient>>) -> Result<Self> {
        let data = block_on(map_torrent_data(&client, 1))?;
        Ok(Self {
            client,
            data,
            selected_tab: SelectedTab::Info,
            colors: Colors::new(),
        })
    }

    pub fn next_tab(&mut self) {
        self.selected_tab = self.selected_tab.next();
    }

    pub fn previous_tab(&mut self) {
        self.selected_tab = self.selected_tab.previous();
    }

    fn render_tabs(&self, frame: &mut Frame, area: Rect) {
        let titles = SelectedTab::iter().map(SelectedTab::title);
        let highlight_style = Style::default()
            .add_modifier(Modifier::REVERSED)
            .fg(self.colors.tab_selected);
        let selected_tab_index = self.selected_tab as usize;
        let tabs = Tabs::new(titles)
            .highlight_style(highlight_style)
            .select(selected_tab_index)
            .padding("", "")
            .divider(" ");

        let rects = Layout::vertical([Constraint::Min(2), Constraint::Percentage(100)]).split(area);

        frame.render_widget(tabs, rects[0]);
        match self.selected_tab {
            SelectedTab::Info => InfoTab::new(&self.data).render(frame, rects[1]),
            SelectedTab::Peers => PeersTab::new(&self.data).render(frame, rects[1]),
            SelectedTab::Tracker => TrackersTab::new(&self.data).render(frame, rects[1]),
            SelectedTab::Files => FilesTab::new(&self.data).render(frame, rects[1]),
        }
    }
}

async fn map_torrent_data(
    client: &Rc<RefCell<TransClient>>,
    id: i64,
) -> Result<TorrentData, AppError> {
    let res = {
        let mut client = client.borrow_mut();
        async move { client.torrent_get(None, Some(vec![Id::Id(id)])).await }
    }
    .await;

    let torrents = match res {
        Ok(t) => t.arguments.torrents,
        Err(err) => return Err(AppError::WithMessage(err.to_string())),
    };

    torrents
        .iter()
        .filter_map(|t| {
            let t = t.clone();
            let trackers = t
                .tracker_stats?
                .iter()
                .map(|tr| TrackerData {
                    host: tr.host.to_string(),
                    is_backup: tr.is_backup,
                    next_announce: tr.next_announce_time,
                })
                .collect_vec();
            let files = t
                .files?
                .iter()
                .enumerate()
                .filter_map(|(i, f)| {
                    let file_stats = t.file_stats.clone()?;
                    Some(FilesData {
                        name: f.name.to_string(),
                        downloaded: convert_bytes(f.bytes_completed),
                        total_size: convert_bytes(f.length),
                        priority: convert_priority(file_stats.get(i)?.priority.clone()),
                        wanted: file_stats.get(i)?.wanted,
                    })
                })
                .collect_vec();

            Some(TorrentData {
                is_stalled: t.is_stalled?,
                name: t.name?,
                eta: convert_eta(t.eta?),
                ratio: handle_ratio(t.upload_ratio?),
                percent_done: convert_percentage(t.percent_done?),
                total_size: convert_bytes(t.total_size?),
                size_done: convert_bytes(t.size_when_done?),
                uploaded: convert_bytes(t.uploaded_ever?),
                downloaded: convert_bytes(t.size_when_done? - t.left_until_done?),
                location: t.download_dir?,
                hash: t.hash_string?,
                added_date: DateTime::from_timestamp(t.added_date?, 0)?,
                done_date: DateTime::from_timestamp(t.done_date?, 0)?,
                error: t.error_string?,
                trackers,
                files,
            })
        })
        .next()
        .ok_or(AppError::OutOfBound)
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

pub struct TorrentData {
    is_stalled: bool,
    name: String,
    percent_done: String,
    total_size: String,
    size_done: String,
    uploaded: String,
    downloaded: String,
    ratio: String,
    location: String,
    hash: String,
    added_date: DateTime<Utc>,
    done_date: DateTime<Utc>,
    eta: String,
    error: String,

    trackers: Vec<TrackerData>,
    files: Vec<FilesData>,
}

pub struct TrackerData {
    host: String,
    is_backup: bool,
    next_announce: DateTime<Utc>,
}

pub struct FilesData {
    name: String,
    downloaded: String,
    total_size: String,
    priority: String,
    wanted: bool,
}
