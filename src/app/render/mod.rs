use color_eyre::eyre::Context;
use tui::style::Color;
use tui::text::{Span, Spans};
use tui::widgets::*;
use tui::{layout::Corner, style::Style};
use twitch_irc::message::PrivmsgMessage;

use super::model::*;
use super::{Backend, Terminal};

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
                ChatItem::Message(msg) => ListItem::new(self.render_message(msg)),
                ChatItem::Event(msg) => ListItem::new(msg.to_string()),
            })
            .collect::<Vec<_>>();
        List::new(chat)
            .block(Block::default().title("Chat").borders(Borders::all()))
            .highlight_style(Style::default())
            .start_corner(Corner::BottomLeft)
    }

    fn render_message<'a>(&self, msg: &'a PrivmsgMessage) -> Spans<'a> {
        let color = msg
            .name_color
            .map_or(Color::Blue, |color| Color::Rgb(color.r, color.g, color.b));
        Spans::from(vec![
            Span::styled(&msg.sender.name, Style::default().fg(color)),
            Span::raw(": "),
            Span::raw(&msg.message_text),
        ])
    }
}
