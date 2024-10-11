use itertools::Itertools;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    widgets::Block,
    Frame,
};
use tui_tree_widget::{Tree, TreeItem, TreeState};

use crate::colors::Colors;

use super::TorrentData;

pub struct FilesTab {
    data: TorrentData,
    state: TreeState<String>,
    colors: Colors,
}

enum Node {
    Directory(String, Box<Node>),
    File(String),
}
use Node::*;

impl FilesTab {
    pub fn new(data: &TorrentData) -> Self {
        Self {
            data: data.clone(),
            state: TreeState::default(),
            colors: Colors::new(),
        }
    }

    pub fn down(&mut self) {
        self.state.key_down();
    }
    pub fn up(&mut self) {
        self.state.key_up();
    }

    pub fn scroll_up(&mut self, amount: usize) {
        self.state.scroll_up(amount);
    }
    pub fn scroll_down(&mut self, amount: usize) {
        self.state.scroll_up(amount);
    }

    pub fn top(&mut self) {
        self.state.select_first();
    }
    pub fn bottom(&mut self) {
        self.state.select_last();
    }

    pub fn toggle(&mut self) {
        self.state.toggle_selected();
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let rects = Layout::vertical([Constraint::Min(5), Constraint::Length(3)]).split(area);
        let file_style = Style::default()
            .fg(self.colors.row_fg)
            .bg(self.colors.buffer_bg);
        let border_style = Style::default().fg(self.colors.footer_border_color);
        let selected_style = Style::default()
            .add_modifier(Modifier::REVERSED)
            .fg(self.colors.selected_style_fg);

        let items = self
            .data
            .files
            .iter()
            .map(|file| {
                let mut paths = file.name.split('/').rev().collect_vec();
                let len = paths.len();
                map_node(&parse_node(&mut paths, len))
            })
            .collect_vec();

        let tree = Tree::new(&items)
            .expect("unique identifier")
            .style(file_style)
            .highlight_style(selected_style)
            .block(Block::bordered().border_style(border_style));

        frame.render_stateful_widget(tree, rects[0], &mut self.state);
    }
}

fn map_node(node: &Node) -> TreeItem<'static, String> {
    let mut children = Vec::new();
    match node {
        Directory(name, node) => {
            children.push(map_node(node));
            TreeItem::new(name.to_string(), name.to_string(), children).expect("unique identifier")
        }
        File(name) => TreeItem::new_leaf(name.to_string(), name.to_string()),
    }
}

fn parse_node(input: &mut Vec<&str>, len: usize) -> Node {
    if len > 1 {
        let dir_name = input.pop().unwrap();
        Directory(
            dir_name.to_string(),
            Box::new(parse_node(input, input.len())),
        )
    } else {
        let file_name = input.first().unwrap();
        File(file_name.to_string())
    }
}
