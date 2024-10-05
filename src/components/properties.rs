use std::{cell::RefCell, rc::Rc};

use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use files::FilesTab;
use futures::executor::block_on;
use info::InfoTab;
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
use transmission_rpc::TransClient;

use crate::{
    action::Action,
    app::{AppError, Mode},
    colors::Colors,
    data::{map_torrent_data, TorrentData},
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
                self.data = match block_on(map_torrent_data(&self.client, Some(self.data.id))) {
                    Ok(d) => d.first().ok_or(AppError::OutOfBound)?.clone(),
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
            KeyCode::Esc | KeyCode::Backspace => {
                return Ok(Some(Action::Mode(Mode::Home, self.data.id)));
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
    pub fn new(client: Rc<RefCell<TransClient>>, id: i64) -> Result<Self> {
        let data = block_on(map_torrent_data(&client, Some(id)))?
            .first()
            .ok_or(AppError::OutOfBound)?
            .clone();
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
