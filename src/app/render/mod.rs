use color_eyre::eyre::Context;
use tui::{layout::Corner, style::Style};

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
            .draw(|frame| draw_frame(model, frame))
            .wrap_err("when rendering terminal")?;
        Ok(())
    }
}

fn draw_frame(model: &Model, frame: &mut Frame) {
    use tui::widgets::*;

    let size = frame.size();
    let chat = model
        .chat
        .iter()
        .rev() // Reverse to show newest at the bottom
        .map(|item| match item {
            ChatItem::Message(msg) => {
                ListItem::new(format!("{}: {}", msg.sender.name, msg.message_text))
            }
            ChatItem::Event(msg) => ListItem::new(msg.to_string()),
        })
        .collect::<Vec<_>>();
    let chat = List::new(chat)
        .block(Block::default().title("Chat").borders(Borders::all()))
        .highlight_style(Style::default())
        .start_corner(Corner::BottomLeft);
    frame.render_widget(chat, size);
}
