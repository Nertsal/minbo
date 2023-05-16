use tui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Span, Spans, StyledGrapheme, Text},
    widgets::{Block, Borders, Widget},
};
use twitch_irc::message::PrivmsgMessage;
use unicode_width::UnicodeWidthStr;

use crate::app::model::{ChatItem, Model};

use super::Render;

const NAME_LENGTH: usize = 25;

impl Render {
    pub fn render_chat<'a>(&self, model: &'a Model) -> impl Widget + 'a {
        let chat: Vec<_> = model
            .chat
            .iter()
            .rev() // Reverse to show newest at the bottom
            .map(|item| match item {
                ChatItem::Message(msg) => self.render_message(model, msg),
                ChatItem::Event(msg) => self.render_event(msg),
            })
            .collect();
        Chat::new(chat, Some(0)).block(Block::default().title("Chat").borders(Borders::all()))
    }

    fn render_event<'a>(&self, msg: &'a str) -> ChatItemRender<'a> {
        let spans = super::colorize_names(msg, &self.chatters);
        let mut text = Text::from(Spans::from(spans));
        text.patch_style(Style::default().fg(Color::Magenta).bg(Color::DarkGray));
        ChatItemRender::Event { text }
    }

    fn render_message<'a>(&self, model: &Model, msg: &'a PrivmsgMessage) -> ChatItemRender<'a> {
        let color = model
            .chatters
            .get(&msg.sender.name)
            .copied()
            .unwrap_or(Color::LightBlue);
        let sender = Spans::from(vec![Span::styled(
            &msg.sender.name,
            Style::default().fg(color),
        )]);
        let msg = Spans::from(super::colorize_names(&msg.message_text, &self.chatters));
        ChatItemRender::Msg { sender, msg }
    }
}

fn get_line_offset(line_width: u16, text_area_width: u16, alignment: Alignment) -> u16 {
    match alignment {
        Alignment::Center => (text_area_width / 2).saturating_sub(line_width / 2),
        Alignment::Right => text_area_width.saturating_sub(line_width),
        Alignment::Left => 0,
    }
}

#[derive(Debug, Clone)]
enum ChatItemRender<'a> {
    Msg { sender: Spans<'a>, msg: Spans<'a> },
    Event { text: Text<'a> },
}

#[derive(Debug, Clone)]
struct Chat<'a> {
    /// A block to wrap the widget in
    block: Option<Block<'a>>,
    /// Widget style
    style: Style,
    /// Chat messages/events
    items: Vec<ChatItemRender<'a>>,
    /// Index of the selected message. Events cannot be selected.
    selected_message: Option<usize>,
}

impl<'a> Chat<'a> {
    pub fn new(items: Vec<ChatItemRender<'a>>, selected_message: Option<usize>) -> Chat<'a> {
        Chat {
            block: None,
            style: Style::default(),
            items,
            selected_message,
        }
    }

    pub fn block(mut self, block: Block<'a>) -> Chat<'a> {
        self.block = Some(block);
        self
    }

    pub fn style(mut self, style: Style) -> Chat<'a> {
        self.style = style;
        self
    }
}

impl<'a> Widget for Chat<'a> {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        buf.set_style(area, self.style);
        let text_area = match self.block.take() {
            Some(b) => {
                let inner_area = b.inner(area);
                b.render(area, buf);
                inner_area
            }
            None => area,
        };

        if text_area.height < 1 {
            return;
        }

        // TODO
        let highlight_symbol = ">";
        let blank_symbol = " ".repeat(highlight_symbol.width());

        // TODO: calculate
        let msg_max_width = 40;
        let event_max_width = 50;

        // Wrap all lines
        let mut lines = Vec::new();
        for item in &self.items {
            match item {
                ChatItemRender::Msg { sender, msg } => {
                    // TODO: sender
                    lines.extend(wrap_spans(msg, msg_max_width));
                }
                ChatItemRender::Event { text } => {
                    for line in &text.lines {
                        lines.extend(wrap_spans(line, event_max_width));
                    }
                }
            }
        }

        // Render line by line
        let mut y = 0;
        for current_line in lines {
            let mut x = 0;
            // Grapheme by grapheme
            for StyledGrapheme { symbol, style } in current_line
                .0
                .iter()
                .flat_map(|span| span.styled_graphemes(self.style))
            {
                buf.get_mut(text_area.left() + x, text_area.bottom() - 1 - y)
                    .set_symbol(if symbol.is_empty() {
                        // If the symbol is empty, the last char which rendered last time will
                        // leave on the line. It's a quick fix.
                        " "
                    } else {
                        symbol
                    })
                    .set_style(style);
                x += symbol.width() as u16;
            }

            y += 1;
            if y >= text_area.height {
                // Text does not fit
                break;
            }
        }
    }
}

fn wrap_spans<'a>(spans: &Spans<'a>, max_width: u16) -> Vec<Spans<'a>> {
    vec![spans.clone()]
}
