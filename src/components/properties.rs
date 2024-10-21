use std::{cell::RefCell, rc::Rc};

use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use futures::executor::block_on;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{palette::tailwind, Modifier, Style, Stylize},
    text::Line,
    widgets::Tabs,
    Frame,
};
use strum::{Display, EnumIter, FromRepr, IntoEnumIterator};
use transmission_rpc::TransClient;

use crate::{
    action::Action,
    app::{self, Mode},
    colors::Colors,
    data::{self, map_torrent_data},
};

use super::Component;

const SCROLL_SIZE: usize = 4;

pub mod files;
pub mod info;
pub mod peers;
pub mod trackers;

pub struct Properties {
    client: Rc<RefCell<TransClient>>,
    data: data::Torrent,
    selected_tab: SelectedTab,
    info_tab: info::Tab,
    tracker_tab: trackers::Tab,
    files_tab: files::Tab,
    colors: Colors,
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Display, FromRepr, EnumIter)]
enum SelectedTab {
    #[default]
    #[strum(to_string = "Info")]
    Info,
    // #[strum(to_string = "Peers")]
    // Peers,
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
                    Ok(d) => d.first().ok_or(app::Error::OutOfBound)?.clone(),
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
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.next();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.previous();
            }
            KeyCode::Char('g') | KeyCode::Home => {
                self.top();
            }
            KeyCode::Char('G') | KeyCode::End => {
                self.bottom();
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.scroll_up(SCROLL_SIZE);
            }
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.scroll_down(SCROLL_SIZE);
            }
            KeyCode::Enter => {
                if self.selected_tab == SelectedTab::Files {
                    self.files_tab.toggle();
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
            .ok_or(app::Error::OutOfBound)?
            .clone();
        Ok(Self {
            client,
            info_tab: info::Tab::new(&data),
            tracker_tab: trackers::Tab::new(&data),
            files_tab: files::Tab::new(&data),
            data,
            selected_tab: SelectedTab::Info,
            colors: Colors::new(),
        })
    }

    fn next_tab(&mut self) {
        self.selected_tab = self.selected_tab.next();
    }

    fn previous_tab(&mut self) {
        self.selected_tab = self.selected_tab.previous();
    }

    fn next(&mut self) {
        match self.selected_tab {
            SelectedTab::Tracker => self.tracker_tab.next(),
            SelectedTab::Files => self.files_tab.down(),
            _ => {}
        }
    }

    fn previous(&mut self) {
        match self.selected_tab {
            SelectedTab::Tracker => self.tracker_tab.previous(),
            SelectedTab::Files => self.files_tab.up(),
            _ => {}
        }
    }

    fn top(&mut self) {
        match self.selected_tab {
            SelectedTab::Tracker => self.tracker_tab.top(),
            SelectedTab::Files => self.files_tab.top(),
            _ => {}
        }
    }

    fn bottom(&mut self) {
        match self.selected_tab {
            SelectedTab::Tracker => self.tracker_tab.bottom(),
            SelectedTab::Files => self.files_tab.bottom(),
            _ => {}
        }
    }

    fn scroll_down(&mut self, amount: usize) {
        match self.selected_tab {
            SelectedTab::Tracker => self.tracker_tab.scroll_down(amount),
            SelectedTab::Files => self.files_tab.scroll_down(amount),
            _ => {}
        }
    }

    fn scroll_up(&mut self, amount: usize) {
        match self.selected_tab {
            SelectedTab::Tracker => self.tracker_tab.scroll_up(amount),
            SelectedTab::Files => self.files_tab.scroll_up(amount),
            _ => {}
        }
    }

    fn render_tabs(&mut self, frame: &mut Frame, area: Rect) {
        let titles = SelectedTab::iter().map(SelectedTab::title);
        let highlight_style = Style::default()
            .add_modifier(Modifier::REVERSED)
            .fg(self.colors.tab_selected);
        let selected_tab_index = self.selected_tab as usize;
        let tabs = Tabs::new(titles)
            .highlight_style(highlight_style)
            .select(selected_tab_index)
            .bg(self.colors.buffer_bg)
            .padding("", "")
            .divider(" ");

        let rects = Layout::vertical([Constraint::Min(1), Constraint::Percentage(100)]).split(area);

        frame.render_widget(tabs, rects[0]);
        match self.selected_tab {
            SelectedTab::Info => self.info_tab.render(frame, rects[1]),
            SelectedTab::Tracker => self.tracker_tab.render(frame, rects[1]),
            SelectedTab::Files => self.files_tab.render(frame, rects[1]),
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
