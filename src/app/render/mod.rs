use color_eyre::eyre::Context;
use tui::style::Color;
use tui::text::{Span, Spans};
use tui::widgets::*;
use tui::{layout::Corner, style::Style};
use twitch_irc::message::PrivmsgMessage;

use super::model::*;
use super::{Backend, Terminal};

const NAME_LENGTH: usize = 25;

type Frame<'a> = tui::Frame<'a, Backend>;

pub struct Render {}

impl Render {
    pub fn new() -> Self {
        Self {}
    }

    /// Render the model to the terminal.
    pub fn draw(&mut self, terminal: &mut Terminal, model: &Model) -> color_eyre::Result<()> {
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
        let chat = model
            .chat
            .iter()
            .rev() // Reverse to show newest at the bottom
            .map(|item| match item {
                ChatItem::Message(msg) => ListItem::new(self.render_message(model, msg)),
                ChatItem::Event(msg) => ListItem::new(msg.to_string()),
            })
            .collect::<Vec<_>>();
        List::new(chat)
            .block(Block::default().title("Chat").borders(Borders::all()))
            .highlight_style(Style::default())
            .start_corner(Corner::BottomLeft)
    }

    fn render_message<'a>(&self, model: &Model, msg: &'a PrivmsgMessage) -> Spans<'a> {
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
        spans.extend(colorize_names(
            &msg.message_text,
            model
                .chatters
                .iter()
                .map(|(name, &color)| (name.clone(), color))
                .collect(),
        ));
        Spans::from(spans)
    }
}

/// Find all names in the message and
fn colorize_names(message: &str, mut names: Vec<(String, Color)>) -> Vec<Span<'_>> {
    enum Slice<'a> {
        Raw(&'a str),
        Colored(Span<'a>),
    }

    let mut result = vec![Slice::Raw(message)];

    // Process longest names first in case someone's name is a substring of another person's name.
    // That way the longest name will be prioritized.
    names.sort_by_key(|(name, _)| std::cmp::Reverse(name.len()));
    for (name, color) in names {
        let name = name.to_lowercase();
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
