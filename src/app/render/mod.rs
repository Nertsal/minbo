mod chat;

use super::{Backend, Terminal};

use color_eyre::eyre::Context;
use tui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders},
};
use tui_logger::TuiLoggerWidget;

use crate::model::*;

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
            .chat
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
        // Layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(30), Constraint::Min(10)].as_ref())
            .split(frame.size());

        let chat = self.render_chat(&model.chat);
        frame.render_widget(chat, chunks[0]);

        let logs = self.render_logs();
        frame.render_widget(logs, chunks[1]);
    }

    fn render_logs(&self) -> TuiLoggerWidget {
        TuiLoggerWidget::default()
            .style_error(Style::default().fg(Color::Red))
            .style_debug(Style::default().fg(Color::Green))
            .style_warn(Style::default().fg(Color::Yellow))
            .style_trace(Style::default().fg(Color::Gray))
            .style_info(Style::default().fg(Color::Blue))
            .block(
                Block::default()
                    .title("Logs")
                    .border_style(Style::default().fg(Color::White).bg(Color::Black))
                    .borders(Borders::ALL),
            )
            .style(Style::default().fg(Color::White).bg(Color::Black))
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
