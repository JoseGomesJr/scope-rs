use crate::interface::DataOut;
use crate::view::View;
use chrono::{DateTime, Local};
use std::fmt::Write;
use std::marker::PhantomData;
use tui::backend::Backend;
use tui::layout::Rect;
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, BorderType, Borders, Paragraph};
use tui::Frame;

pub struct TextView<'a, B: Backend> {
    history: Vec<ViewData<'a>>,
    capacity: usize,
    _marker: PhantomData<B>,
    auto_scroll: bool,
    scroll: (u16, u16),
    frame_height: u16,
}

impl<'a, B: Backend> TextView<'a, B> {
    pub fn new(capacity: usize) -> Self {
        Self {
            history: vec![],
            capacity,
            _marker: PhantomData,
            auto_scroll: true,
            scroll: (0, 0),
            frame_height: u16::MAX,
        }
    }

    fn max_main_axis(&self) -> u16 {
        let main_axis_length = self.frame_height - 5;
        let history_len = self.history.len() as u16;

        if history_len > main_axis_length {
            history_len - main_axis_length
        } else {
            0
        }
    }

    fn is_visible(x: char) -> bool {
        0x20 <= x as u8 && x as u8 <= 0x7E
    }

    fn print_invisible(data: String) -> String {
        data.chars()
            .map(|x| {
                if TextView::<B>::is_visible(x) {
                    x.to_string()
                } else if x == '\n' {
                    "\\n".to_string()
                } else {
                    format!("\\x{:02x}", x as u8)
                }
            })
            .collect::<Vec<String>>()
            .join("")
    }

    fn highlight_invisible(in_text: &str, color: Color) -> Vec<(String, Color)> {
        #[derive(PartialEq)]
        enum Mode {
            Visible,
            Invisible,
        }

        let highlight_color = if color == Color::Magenta {
            Color::Cyan
        } else {
            Color::LightMagenta
        };
        let mut output = vec![];
        let mut text = "".to_string();
        let mut highlight_text = "".to_string();
        let mut mode = Mode::Visible;

        for ch in in_text.chars() {
            if TextView::<B>::is_visible(ch) {
                text.push(ch);
                if mode == Mode::Invisible {
                    output.push((
                        TextView::<B>::print_invisible(highlight_text.clone()),
                        highlight_color,
                    ));
                    highlight_text.clear();
                }
                mode = Mode::Visible;
            } else {
                highlight_text.push(ch);
                if mode == Mode::Visible {
                    output.push((text.clone(), color));
                    text.clear();
                }
                mode = Mode::Invisible;
            }
        }

        if !text.is_empty() {
            output.push((text.clone(), color));
        } else if !highlight_text.is_empty() {
            output.push((
                TextView::<B>::print_invisible(highlight_text.clone()),
                highlight_color,
            ));
        }

        output
    }
}

impl<'a, B: Backend> View for TextView<'a, B> {
    type Backend = B;

    fn draw(&self, f: &mut Frame<Self::Backend>, rect: Rect) {
        let scroll = if self.auto_scroll {
            (self.max_main_axis(), self.scroll.1)
        } else {
            self.scroll
        };

        let (coll, max, coll_size) = (
            &self.history[(scroll.0 as usize)..],
            "".to_string(),
            self.history.len(),
        );

        let block = if self.auto_scroll {
            Block::default()
                .title(format!("[{:03}{}] Text UTF-8", coll_size, max))
                .borders(Borders::ALL)
                .border_type(BorderType::Thick)
                .border_style(Style::default().fg(Color::White))
        } else {
            Block::default()
                .title(format!("[{:03}{}] Text UTF-8", coll_size, max))
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .border_style(
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::RAPID_BLINK),
                )
        };

        let text =
            coll.iter()
                .map(|x| {
                    let scroll = scroll.1 as usize;
                    let content = if scroll >= x.data.len() {
                        ""
                    } else {
                        &x.data[scroll..]
                    };

                    let texts_colors = TextView::<B>::highlight_invisible(content, x.fg);
                    let mut content = vec![x.timestamp.clone()];

                    content.extend(texts_colors.into_iter().map(|(text, color)| {
                        Span::styled(text, Style::default().bg(x.bg).fg(color))
                    }));

                    Spans::from(content)
                })
                .collect::<Vec<_>>();
        let paragraph = Paragraph::new(text).block(block);
        f.render_widget(paragraph, rect);
    }

    fn add_data_out(&mut self, data: DataOut) {
        if self.history.len() >= self.capacity {
            self.history.remove(0);
        }

        match data {
            DataOut::Data(timestamp, data) => {
                let contents = ViewData::decode_ansi_color(&data);
                for (content, color) in contents {
                    self.history
                        .push(ViewData::if_data(timestamp, content, color));
                }
            }
            DataOut::ConfirmData(timestamp, data) => {
                self.history.push(ViewData::user_data(timestamp, data))
            }
            DataOut::ConfirmCommand(timestamp, cmd_name, data) => self
                .history
                .push(ViewData::user_command(timestamp, cmd_name, data)),
            DataOut::ConfirmHexString(timestamp, bytes) => self
                .history
                .push(ViewData::user_hex_string(timestamp, bytes)),
            DataOut::FailData(timestamp, data) => {
                self.history.push(ViewData::fail_data(timestamp, data))
            }
            DataOut::FailCommand(timestamp, cmd_name, _data) => self
                .history
                .push(ViewData::fail_command(timestamp, cmd_name)),
            DataOut::FailHexString(timestamp, bytes) => self
                .history
                .push(ViewData::fail_hex_string(timestamp, bytes)),
        };
    }

    fn clear(&mut self) {
        self.scroll = (0, 0);
        self.auto_scroll = true;
        self.history.clear();
    }

    fn up_scroll(&mut self) {
        if self.max_main_axis() > 0 {
            self.auto_scroll = false;
        }

        if self.scroll.0 < 3 {
            self.scroll.0 = 0;
        } else {
            self.scroll.0 -= 3;
        }
    }

    fn down_scroll(&mut self) {
        let max_main_axis = self.max_main_axis();

        self.scroll.0 += 3;
        self.scroll.0 = self.scroll.0.clamp(0, max_main_axis);

        if self.scroll.0 == max_main_axis {
            self.auto_scroll = true;
        }
    }

    fn left_scroll(&mut self) {
        if self.scroll.1 < 3 {
            self.scroll.1 = 0;
        } else {
            self.scroll.1 -= 3;
        }
    }

    fn right_scroll(&mut self) {
        self.scroll.1 += 3;
    }

    fn set_frame_height(&mut self, frame_height: u16) {
        self.frame_height = frame_height;
    }

    fn update_scroll(&mut self) {
        self.scroll = if self.auto_scroll {
            (self.max_main_axis(), self.scroll.1)
        } else {
            self.scroll
        };
    }
}

#[derive(Clone)]
struct ViewData<'a> {
    timestamp: Span<'a>,
    data: String,
    fg: Color,
    bg: Color,
}

impl<'a> ViewData<'a> {
    fn decode_ansi_color(text: &str) -> Vec<(String, Color)> {
        if text.is_empty() {
            return vec![];
        }

        let splitted = text.split("\x1B[").collect::<Vec<_>>();
        let mut res = vec![];

        let pattern_n_color = [
            ("0m", Color::White),
            ("30m", Color::Black),
            ("0;30m", Color::Black),
            ("31m", Color::Red),
            ("0;31m", Color::Red),
            ("32m", Color::Green),
            ("0;32m", Color::Green),
            ("33m", Color::Yellow),
            ("0;33m", Color::Yellow),
            ("34m", Color::Blue),
            ("0;34m", Color::Blue),
            ("35m", Color::Magenta),
            ("0;35m", Color::Magenta),
            ("36m", Color::Cyan),
            ("0;36m", Color::Cyan),
            ("37m", Color::Gray),
            ("0;37m", Color::Gray),
        ];

        for splitted_str in splitted.iter() {
            if splitted_str.is_empty() {
                continue;
            }

            if pattern_n_color.iter().all(|(pattern, color)| {
                if splitted_str.starts_with(pattern) {
                    let final_str = splitted_str
                        .to_string()
                        .replace(pattern, "")
                        .trim()
                        .to_string();
                    if final_str.is_empty() {
                        return true;
                    }

                    res.push((final_str, *color));
                    return false;
                }

                true
            }) && !splitted_str.starts_with("0m")
            {
                res.push((splitted_str.to_string(), Color::White));
            }
        }

        res
    }

    fn bytes_to_hex_string(bytes: &[u8]) -> String {
        let mut hex_string = String::new();

        for byte in bytes {
            write!(&mut hex_string, "{:02X}", byte).unwrap();
        }

        hex_string
    }

    fn build_timestmap_span(timestamp: DateTime<Local>, fg: Color, bg: Color) -> Span<'a> {
        let tm_fg = if bg != Color::Reset { bg } else { fg };

        Span::styled(
            format!("[{}] ", timestamp.format("%d/%m/%Y %H:%M:%S")),
            Style::default().fg(tm_fg),
        )
    }

    fn if_data(timestamp: DateTime<Local>, content: String, color: Color) -> Self {
        Self {
            timestamp: ViewData::build_timestmap_span(timestamp, color, Color::Reset),
            data: content,
            fg: color,
            bg: Color::Reset,
        }
    }

    fn user_data(timestamp: DateTime<Local>, content: String) -> Self {
        Self {
            timestamp: ViewData::build_timestmap_span(timestamp, Color::Black, Color::LightCyan),
            data: content,
            fg: Color::Black,
            bg: Color::LightCyan,
        }
    }

    fn user_command(timestamp: DateTime<Local>, cmd_name: String, content: String) -> Self {
        let content = format!("</{}> {}", cmd_name, content);

        Self {
            timestamp: ViewData::build_timestmap_span(timestamp, Color::Black, Color::LightGreen),
            data: content,
            fg: Color::Black,
            bg: Color::LightGreen,
        }
    }

    fn user_hex_string(timestamp: DateTime<Local>, bytes: Vec<u8>) -> Self {
        let content = format!("<${}> {:?}", ViewData::bytes_to_hex_string(&bytes), &bytes);

        Self {
            timestamp: ViewData::build_timestmap_span(timestamp, Color::Black, Color::Yellow),
            data: content,
            fg: Color::Black,
            bg: Color::Yellow,
        }
    }

    fn fail_data(timestamp: DateTime<Local>, content: String) -> Self {
        let content = format!("Cannot send \"{}\"", content);

        Self {
            timestamp: ViewData::build_timestmap_span(timestamp, Color::White, Color::LightRed),
            data: content,
            fg: Color::White,
            bg: Color::LightRed,
        }
    }

    fn fail_command(timestamp: DateTime<Local>, cmd_name: String) -> Self {
        let content = format!("Cannot send </{}>", cmd_name);

        Self {
            timestamp: ViewData::build_timestmap_span(timestamp, Color::White, Color::LightRed),
            data: content,
            fg: Color::White,
            bg: Color::LightRed,
        }
    }

    fn fail_hex_string(timestamp: DateTime<Local>, bytes: Vec<u8>) -> Self {
        let content = format!("Cannot send <${}>", ViewData::bytes_to_hex_string(&bytes));

        Self {
            timestamp: ViewData::build_timestmap_span(timestamp, Color::White, Color::LightRed),
            data: content,
            fg: Color::White,
            bg: Color::LightRed,
        }
    }
}
