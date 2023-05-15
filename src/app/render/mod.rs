use color_eyre::eyre::Context;
use tui::style::Color;
use tui::style::Style;
use tui::text::{Span, Spans, Text};
use tui::widgets::*;
use twitch_irc::message::PrivmsgMessage;

use super::model::*;
use super::{Backend, Terminal};

const NAME_LENGTH: usize = 25;

type Frame<'a> = tui::Frame<'a, Backend>;

pub struct Render {
    /// Converted from `model.chatters` for convenience (and to avoid recalculations per widget).
    chatters: Vec<(String, Color)>,
}

impl Render {
    pub fn new() -> Self {
        Self { chatters: vec![] }
    }

    /// Render the model to the terminal.
    pub fn draw(&mut self, terminal: &mut Terminal, model: &Model) -> color_eyre::Result<()> {
        self.chatters = model
            .chatters
            .iter()
            .map(|(name, &color)| (name.clone(), color))
            .collect();
        // Process longest names first in case someone's name is a substring of another person's name.
        // That way the longest name will be prioritized.
        self.chatters
            .sort_by_key(|(name, _)| std::cmp::Reverse(name.len()));

        terminal
            .draw(|frame| self.draw_frame(model, frame))
            .wrap_err("when rendering terminal")?;
        Ok(())
    }

    /// Draw the whole frame.
    fn draw_frame(&self, model: &Model, frame: &mut Frame) {
        let size = frame.size();
        let chat = self.render_chat(model);
        frame.render_widget(chat, size);
    }

    fn render_chat<'a>(&self, model: &'a Model) -> impl Widget + 'a {
        let mut chat = Text::default();
        for item in model
            .chat
            .iter()
            .rev() // Reverse to show newest at the bottom
            .map(|item| match item {
                ChatItem::Message(msg) => self.render_message(model, msg),
                ChatItem::Event(msg) => self.render_event(msg),
            })
        {
            chat.extend(item);
        }
        Paragraph::new(chat)
            .block(Block::default().title("Chat").borders(Borders::all()))
            .wrap(Wrap { trim: false })
    }

    fn render_event<'a>(&self, msg: &'a str) -> Text<'a> {
        let mut spans = vec![Span::raw(format!("{:>w$}: ", "Event", w = NAME_LENGTH))];
        spans.extend(colorize_names(msg, &self.chatters));
        let mut text = Text::from(Spans::from(spans));
        text.patch_style(Style::default().fg(Color::Magenta).bg(Color::DarkGray));
        text
    }

    fn render_message<'a>(&self, model: &Model, msg: &'a PrivmsgMessage) -> Text<'a> {
        let color = model
            .chatters
            .get(&msg.sender.name)
            .copied()
            .unwrap_or(Color::LightBlue);
        let mut spans = vec![
            Span::styled(
                format!("{:>w$}", msg.sender.name, w = NAME_LENGTH),
                Style::default().fg(color),
            ),
            Span::raw(": "),
        ];
        spans.extend(colorize_names(&msg.message_text, &self.chatters));
        Text::from(Spans::from(spans))
    }
}

/// Find all names in the message and
fn colorize_names<'a>(message: &'a str, names: &[(String, Color)]) -> Vec<Span<'a>> {
    enum Slice<'a> {
        Raw(&'a str),
        Colored(Span<'a>),
    }

    let mut result = vec![Slice::Raw(message)];
    for (name, color) in names {
        let name = name.to_lowercase();
        let &color = color;
        result = result
            .into_iter()
            .flat_map(|slice| match slice {
                Slice::Raw(slice) => find_all_substr(slice, &name)
                    .into_iter()
                    .map(|raw| {
                        if raw.to_lowercase() == name {
                            Slice::Colored(Span::styled(raw, Style::default().fg(color)))
                        } else {
                            Slice::Raw(raw)
                        }
                    })
                    .collect(),
                colored => vec![colored],
            })
            .collect();
    }

    result
        .into_iter()
        .map(|slice| match slice {
            Slice::Raw(raw) => Span::raw(raw),
            Slice::Colored(span) => span,
        })
        .collect::<Vec<_>>()
}

/// Looks case-insensitively for `sub` in `s`.
/// Losslessly splits `s` into substrings.
fn find_all_substr<'a>(mut source: &'a str, sub: &str) -> Vec<&'a str> {
    let lower = source.to_lowercase();
    let mut lower = lower.as_str();
    let sub = sub.to_lowercase();
    let mut result = vec![];

    // Find the next match
    while let Some(i) = lower.find(&sub) {
        result.push(&source[..i]);
        let m = i + sub.len();
        result.push(&source[i..m]);
        lower = &lower[m..];
        source = &source[m..];
    }

    // Push the end of `source`
    if !source.is_empty() {
        result.push(source);
    }

    result
}

#[test]
fn test_find_all_substr() {
    assert_eq!(
        find_all_substr("Very cool string of CoOl words", "cool"),
        vec!["Very ", "cool", " string of ", "CoOl", " words"]
    );
}
