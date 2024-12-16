use color_eyre::Result;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, List, ListItem},
    Frame,
};
use tui_tree_widget::{Tree, TreeItem, TreeState};

use crate::{app, colors::Colors, data};

pub struct Tab {
    data: data::Torrent,
    state: TreeState<String>,
    colors: Colors,
}

impl Tab {
    pub fn new(data: &data::Torrent) -> Self {
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

        let items = map_node(&parse_node(
            self.data
                .files
                .iter()
                .map(|f| {
                    format!(
                        "{}\n{}\n{}\n{}\n{}",
                        f.name, f.downloaded, f.total_size, f.priority, f.wanted
                    )
                })
                .collect(),
        ));

        let tree = Tree::new(&items)
            .expect("unique identifier")
            .style(file_style)
            .highlight_style(selected_style)
            .block(Block::bordered().border_style(border_style));

        frame.render_stateful_widget(tree, rects[0], &mut self.state);
    }
}

fn map_node(nodes: &[Node]) -> Vec<TreeItem<'static, String>> {
    nodes
        .iter()
        .map(|node| match node {
            Node::File(data::Files {
                name,
                downloaded,
                total_size,
                priority,
                wanted,
            }) => TreeItem::new_leaf(
                name.to_string(),
                format!(
                    "{} {} {:>10}  {:>10}  {:>10}",
                    wanted, name, downloaded, total_size, priority
                ),
            ),
            Node::Directory(name, children) => {
                TreeItem::new(name.to_string(), name.to_string(), map_node(children))
                    .expect("unique identifier")
            }
        })
        .collect()
}

#[derive(Debug, Clone)]
enum Node {
    File(data::Files),
    Directory(String, Vec<Node>),
}

fn parse_node(paths: Vec<String>) -> Vec<Node> {
    let mut nodes: Vec<Node> = Vec::new();
    for path in paths {
        let vecs = path.lines().collect::<Vec<&str>>();
        let parts = vecs.first().unwrap().split('/').collect::<Vec<&str>>();
        let data = data::Files {
            name: parts.last().unwrap().to_string(),
            downloaded: vecs.get(1).unwrap().parse().unwrap(),
            total_size: vecs.get(2).unwrap().parse().unwrap(),
            priority: vecs.get(3).unwrap().to_string(),
            wanted: vecs.last().unwrap().parse().unwrap(),
        };
        if !parts.is_empty() {
            let _ = insert_into_tree(&mut nodes, &parts, data);
        }
    }

    nodes
}

fn insert_into_tree(children: &mut Vec<Node>, parts: &[&str], data: data::Files) -> Result<()> {
    let Some((current_part, remaining_parts)) = parts.split_first() else {
        return Ok(());
    };

    if remaining_parts.is_empty() {
        children.push(Node::File(data));
        return Ok(());
    }

    if let Some(existing_dir) = children
        .iter_mut()
        .find(|n| matches!(n, Node::Directory(d_name, _) if d_name == current_part))
    {
        if let Node::Directory(_, children) = existing_dir {
            let _ = insert_into_tree(children, remaining_parts, data);
        };
        Ok(())
    } else {
        let new_dir = Node::Directory((*current_part).to_string(), Vec::new());
        children.push(new_dir);
        if let Node::Directory(_, children) = children.last_mut().ok_or(app::Error::OutOfBound)? {
            let _ = insert_into_tree(children, remaining_parts, data);
        };
        Ok(())
    }
}
