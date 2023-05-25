use tui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, StyledGrapheme, Text},
    widgets::{Block, Borders, Widget},
};
use unicode_width::UnicodeWidthStr;

use crate::model::{Chat, ChatItem, ChatMessage, ChatMode};

use super::Render;

const NAME_LENGTH: usize = 25;

impl Render {
    pub fn render_chat<'a>(&self, chat: &'a Chat) -> impl Widget + 'a {
        let items: Vec<_> = chat
            .items
            .iter()
            .map(|item| match item {
                ChatItem::Message(msg) => self.render_message(chat, msg),
                ChatItem::Event(msg) => self.render_event(msg),
            })
            .collect();

        let input = InputWidget {
            content: &chat.input.content,
            cursor: chat.input.cursor,
        };

        ChatWidget::new(chat.mode, items, chat.selected_item, input)
            .block(Block::default().title("Chat").borders(Borders::all()))
    }

    fn render_event<'a>(&self, msg: &'a str) -> ChatItemRender<'a> {
        let spans = super::colorize_names(msg, &self.chatters);
        let mut text = Text::from(Spans::from(spans));
        text.patch_style(Style::default().fg(Color::Magenta));
        ChatItemRender::Event { text }
    }

    fn render_message<'a>(&self, chat: &Chat, msg: &'a ChatMessage) -> ChatItemRender<'a> {
        let color = chat
            .chatters
            .get(&msg.sender_name)
            .copied()
            .unwrap_or(Color::LightBlue);
        let sender = Spans::from(vec![Span::styled(
            &msg.sender_name,
            Style::default().fg(color),
        )]);
        let msg = Spans::from(super::colorize_names(&msg.text, &self.chatters));
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
struct ChatWidget<'a> {
    mode: ChatMode,
    /// A block to wrap the widget in
    block: Option<Block<'a>>,
    /// Widget style
    style: Style,
    /// Chat messages/events
    items: Vec<ChatItemRender<'a>>,
    /// Index of the selected message. Events cannot be selected.
    selected_message: Option<usize>,
    input: InputWidget<'a>,
}

#[derive(Debug, Clone)]
struct InputWidget<'a> {
    content: &'a str,
    cursor: usize,
}

impl<'a> ChatWidget<'a> {
    pub fn new(
        mode: ChatMode,
        items: Vec<ChatItemRender<'a>>,
        selected_message: Option<usize>,
        input: InputWidget<'a>,
    ) -> ChatWidget<'a> {
        ChatWidget {
            mode,
            block: None,
            style: Style::default(),
            items,
            selected_message,
            input,
        }
    }

    pub fn block(mut self, block: Block<'a>) -> ChatWidget<'a> {
        self.block = Some(block);
        self
    }

    pub fn style(mut self, style: Style) -> ChatWidget<'a> {
        self.style = style;
        self
    }
}

impl<'a> Widget for ChatWidget<'a> {
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
        let msg_prefix_len = NAME_LENGTH + 2 + highlight_symbol.width();
        let msg_max_width = usize::from(area.width).saturating_sub(msg_prefix_len);
        let event_max_width = usize::from(area.width) / 2;

        // Wrap all lines
        let mut chat_lines = Vec::new();
        // TODO: ignore old messages/cache wrapping
        for (item_id, item) in self.items.iter().enumerate() {
            // Highlight prefix
            let prefix = if Some(item_id) == self.selected_message && self.mode == ChatMode::Normal
            {
                highlight_symbol
            } else {
                &blank_symbol
            };
            let prefix = Span::raw(prefix);

            match item {
                ChatItemRender::Msg { sender, msg } => {
                    // Sender prefix
                    let sender_width = sender.width();
                    let mut sender_line = vec![
                        prefix.clone(),
                        Span::raw(" ".repeat(NAME_LENGTH - sender_width)),
                    ];
                    sender_line.extend(sender.0.clone());
                    sender_line.push(Span::raw(": "));

                    // Blank prefix
                    let blank_line = vec![prefix, Span::raw(" ".repeat(NAME_LENGTH + 2))];

                    // Wrap message lines
                    let mut lines = wrap_spans(msg, msg_max_width);

                    // Prefix the first line with sender
                    match lines.get_mut(0) {
                        Some(line) => {
                            let l = std::mem::take(line);
                            sender_line.extend(l.0);
                            *line = Spans::from(sender_line);
                        }
                        None => {
                            lines.push(Spans::from(sender_line));
                        }
                    }

                    // Prefix other lines with space
                    for line in lines.iter_mut().skip(1) {
                        let l = std::mem::take(line);
                        let mut blank_line = blank_line.clone();
                        blank_line.extend(l.0);
                        *line = Spans::from(blank_line);
                    }

                    // Reverse lines since they are rendered in reverse
                    chat_lines.extend(lines.into_iter().map(|line| (line, Alignment::Left)));
                }
                ChatItemRender::Event { text } => {
                    for line in &text.lines {
                        // Wrap each line in the message
                        chat_lines.extend(
                            wrap_spans(line, event_max_width)
                                .into_iter()
                                .map(|line| (line, Alignment::Center)),
                        );
                    }
                }
            }
        }

        // Input box
        if let ChatMode::Insert = self.mode {
            let (before, after) = self
                .input
                .content
                .split_at(self.input.cursor.min(self.input.content.len()));
            let (at, after) = if after.is_empty() {
                (" ", "")
            } else {
                after.split_at(1)
            };

            let spans = Spans::from(vec![
                Span::raw(" ".repeat(NAME_LENGTH + 1)), // +1 for the colon after the name
                Span::raw(highlight_symbol),
                Span::raw(" "),
                Span::raw(before),
                Span::styled(at, Style::default().add_modifier(Modifier::UNDERLINED)),
                Span::raw(after),
            ]);
            chat_lines.push((spans, Alignment::Left));
        }

        // Render line by line in reverse order to show newest first
        let mut y = 0;
        for (current_line, alignment) in chat_lines.into_iter().rev() {
            let mut x = get_line_offset(current_line.width() as u16, area.width, alignment);
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
                if x >= text_area.width {
                    // Text does not fit
                    // log::error!(
                    //     "Text does not fit into area {:?} with alignment: {:?}:\n^-text: {:?}",
                    //     area,
                    //     alignment,
                    //     current_line
                    // );
                    break;
                }
            }

            y += 1;
            if y >= text_area.height {
                // Text does not fit
                break;
            }
        }
    }
}

fn wrap_spans<'a>(spans: &'a Spans<'a>, max_width: usize) -> Vec<Spans<'a>> {
    if max_width == 0 {
        panic!("No space available");
    }

    let mut lines = Vec::new();
    let mut current_line = Vec::new();
    let mut width_left = max_width;

    macro_rules! new_line {
        () => {{
            if !current_line.is_empty() {
                let line = ::std::mem::take(&mut current_line);
                lines.push(::tui::text::Spans::from(line));
            }
            width_left = max_width;
        }};
    }

    macro_rules! push_span {
        ($span:expr) => {{
            let width = $span.width();
            width_left = width_left.saturating_sub(width);
            current_line.push($span);
        }};
    }

    for span in &spans.0 {
        let style = span.style;
        // Go over words
        for word in span.content.split_inclusive(char::is_whitespace) {
            let width = word.width();
            if current_line.is_empty() || width <= width_left {
                // First word or enough space
                push_span!(Span::styled(word, style));
                continue;
            }

            // Not enough space -> new line
            new_line!();
            push_span!(Span::styled(word, style));
        }
    }

    if !current_line.is_empty() {
        lines.push(::tui::text::Spans::from(current_line));
    }

    lines
}
