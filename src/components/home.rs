use crate::utils::{convert_bytes, convert_eta, convert_percentage, convert_status};

use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use futures::executor::block_on;
use itertools::Itertools;
use ratatui::{prelude::*, widgets::*};
use style::palette::tailwind;
use tokio::sync::mpsc::UnboundedSender;
use transmission_rpc::TransClient;
use unicode_width::UnicodeWidthStr;

use super::Component;
use crate::{action::Action, config::Config};

pub struct Home {
    client: TransClient,
    state: TableState,
    items: Vec<Data>,
    longest_item_lens: (u16, u16, u16, u16, u16, u16, u16, u16, u16),
    colors: TableColors,
    color_index: usize,
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
}

impl Home {
    pub fn new(mut client: TransClient) -> Self {
        let data_vec = block_on(get_torrent_data(&mut client)).unwrap();
        Self {
            client,
            state: TableState::default().with_selected(0),
            longest_item_lens: constraint_len_calculator(&data_vec),
            colors: TableColors::new(&PALETTES[0]),
            color_index: 0,
            items: data_vec,
            command_tx: None,
            config: Config::default(),
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn next_color(&mut self) {
        self.color_index = (self.color_index + 1) % PALETTES.len();
    }

    pub fn previous_color(&mut self) {
        let count = PALETTES.len();
        self.color_index = (self.color_index + count - 1) % count;
    }

    pub fn set_colors(&mut self) {
        self.colors = TableColors::new(&PALETTES[self.color_index]);
    }
}

impl Home {
    fn render_table(&mut self, frame: &mut Frame, area: Rect) {
        let header_style = Style::default()
            .fg(self.colors.header_fg)
            .bg(self.colors.header_bg);
        let selected_style = Style::default()
            .add_modifier(Modifier::REVERSED)
            .fg(self.colors.selected_style_fg);

        let header = [
            "Name", "Done", "Size", "Have", "ETA", "Up", "Down", "Ratio", "Status",
        ]
        .into_iter()
        .map(Cell::from)
        .collect::<Row>()
        .style(header_style)
        .height(1);
        let rows = self.items.iter().enumerate().map(|(i, data)| {
            let color = match i % 2 {
                0 => self.colors.normal_row_color,
                _ => self.colors.alt_row_color,
            };
            let item = data.ref_array();
            item.into_iter()
                .map(|content| Cell::from(Text::from(format!("\n{content}\n"))))
                .collect::<Row>()
                .style(Style::new().fg(self.colors.row_fg).bg(color))
                .height(4)
        });
        let bar = " █ ";
        let t = Table::new(
            rows,
            [
                // + 1 is for padding.
                Constraint::Length(self.longest_item_lens.0 + 1),
                Constraint::Min(self.longest_item_lens.1 + 1),
                Constraint::Min(self.longest_item_lens.2 + 1),
                Constraint::Min(self.longest_item_lens.3 + 1),
                Constraint::Min(self.longest_item_lens.4 + 1),
                Constraint::Min(self.longest_item_lens.5 + 1),
                Constraint::Min(self.longest_item_lens.6 + 1),
                Constraint::Min(self.longest_item_lens.7 + 1),
                Constraint::Min(self.longest_item_lens.8),
            ],
        )
        .header(header)
        .highlight_style(selected_style)
        .highlight_symbol(Text::from(vec![
            "".into(),
            bar.into(),
            bar.into(),
            "".into(),
        ]))
        .bg(self.colors.buffer_bg)
        .highlight_spacing(HighlightSpacing::Always);
        frame.render_stateful_widget(t, area, &mut self.state);
    }

    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        let info_footer = Paragraph::new(Line::from(INFO_TEXT))
            .style(
                Style::new()
                    .fg(self.colors.row_fg)
                    .bg(self.colors.buffer_bg),
            )
            .centered()
            .block(
                Block::bordered()
                    .border_type(BorderType::Double)
                    .border_style(Style::new().fg(self.colors.footer_border_color)),
            );
        frame.render_widget(info_footer, area);
    }
}
impl Component for Home {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.config = config;
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<Option<Action>> {
        match key_event.code {
            KeyCode::Right | KeyCode::Char('l') => {
                self.next_color();
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.previous_color();
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.next();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.previous();
            }
            // Other handlers you could add here.
            _ => {}
        }
        Ok(None)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Tick => {
                self.items = block_on(get_torrent_data(&mut self.client))?;
            }
            Action::Render => {}
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let vertical = &Layout::vertical([Constraint::Min(5), Constraint::Length(3)]);
        let rects = vertical.split(area);

        self.set_colors();

        self.render_table(frame, rects[0]);
        self.render_footer(frame, rects[1]);
        Ok(())
    }
}

const PALETTES: [tailwind::Palette; 4] = [
    tailwind::BLUE,
    tailwind::EMERALD,
    tailwind::INDIGO,
    tailwind::RED,
];
const INFO_TEXT: &str =
    "(Esc) quit | (↑) move up | (↓) move down | (→) next color | (←) previous color";

struct TableColors {
    buffer_bg: Color,
    header_bg: Color,
    header_fg: Color,
    row_fg: Color,
    selected_style_fg: Color,
    normal_row_color: Color,
    alt_row_color: Color,
    footer_border_color: Color,
}

impl TableColors {
    const fn new(color: &tailwind::Palette) -> Self {
        Self {
            buffer_bg: tailwind::SLATE.c950,
            header_bg: color.c900,
            header_fg: tailwind::SLATE.c200,
            row_fg: tailwind::SLATE.c200,
            selected_style_fg: color.c400,
            normal_row_color: tailwind::SLATE.c950,
            alt_row_color: tailwind::SLATE.c900,
            footer_border_color: color.c400,
        }
    }
}

struct Data {
    name: String,
    done: String,
    have: String,
    eta: String,
    up: String,
    down: String,
    ratio: String,
    status: String,
    size: String,
}

impl Data {
    const fn ref_array(&self) -> [&String; 9] {
        [
            &self.name,
            &self.done,
            &self.size,
            &self.have,
            &self.eta,
            &self.up,
            &self.down,
            &self.ratio,
            &self.status,
        ]
    }

    fn name(&self) -> &str {
        &self.name
    }
    fn done(&self) -> &str {
        &self.done
    }
    fn have(&self) -> &str {
        &self.have
    }
    fn eta(&self) -> &str {
        &self.eta
    }
    fn up(&self) -> &str {
        &self.up
    }
    fn down(&self) -> &str {
        &self.down
    }
    fn ratio(&self) -> &str {
        &self.ratio
    }
    fn status(&self) -> &str {
        &self.status
    }
    fn size(&self) -> &str {
        &self.size
    }
}

async fn get_torrent_data(client: &mut TransClient) -> Result<Vec<Data>> {
    let res = client.torrent_get(None, None).await.unwrap();

    Ok(res
        .arguments
        .torrents
        .iter()
        .map(|t| {
            let name = t.name.clone().unwrap().to_string();
            let done = convert_percentage(t.percent_done.unwrap());
            let size = convert_bytes(t.size_when_done.unwrap());
            let have = convert_bytes(t.left_until_done.unwrap());
            let eta = convert_eta(t.eta.unwrap());
            let up = format!("{}/s", convert_bytes(t.rate_upload.unwrap()));
            let down = format!("{}/s", convert_bytes(t.rate_download.unwrap()));
            let ratio = t.upload_ratio.unwrap().to_string();
            let status = convert_status(t.status.unwrap());

            Data {
                name,
                done,
                size,
                have,
                eta,
                up,
                down,
                ratio,
                status,
            }
        })
        .sorted_by(|a, b| a.name.cmp(&b.name))
        .collect_vec())
}

fn constraint_len_calculator(items: &[Data]) -> (u16, u16, u16, u16, u16, u16, u16, u16, u16) {
    let name_len = items
        .iter()
        .map(Data::name)
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or(0);
    let done_len = items
        .iter()
        .map(Data::done)
        .flat_map(str::lines)
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or(0);
    let size_len = items
        .iter()
        .map(Data::size)
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or(0);
    let have_len = items
        .iter()
        .map(Data::have)
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or(0);
    let eta_len = items
        .iter()
        .map(Data::eta)
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or(0);
    let up_len = items
        .iter()
        .map(Data::up)
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or(0);
    let down_len = items
        .iter()
        .map(Data::down)
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or(0);
    let ratio_len = items
        .iter()
        .map(Data::ratio)
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or(0);
    let status_len = items
        .iter()
        .map(Data::status)
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or(0);

    #[allow(clippy::cast_possible_truncation)]
    (
        name_len as u16,
        done_len as u16,
        size_len as u16,
        have_len as u16,
        eta_len as u16,
        up_len as u16,
        down_len as u16,
        ratio_len as u16,
        status_len as u16,
    )
}
