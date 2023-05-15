use color_eyre::eyre::Context;
use crossterm::{event::EnableMouseCapture, execute, terminal::EnterAlternateScreen};
use tui::{backend::CrosstermBackend, layout::Corner, style::Style};

use super::model::*;

type Backend = CrosstermBackend<std::io::Stdout>;
type Terminal = tui::Terminal<Backend>;
type Frame<'a> = tui::Frame<'a, Backend>;

pub struct Render {
    terminal: Terminal,
}

impl Render {
    /// Configures the terminal.
    pub fn new() -> color_eyre::Result<Self> {
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
            .wrap_err("when setting up stdout")?;

        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend).wrap_err("when setting up terminal")?;

        crossterm::terminal::enable_raw_mode().wrap_err("when enabling terminal raw mode")?;

        Ok(Self { terminal })
    }

    pub fn draw(&mut self, model: &Model) -> color_eyre::Result<()> {
        self.terminal
            .draw(|frame| draw_frame(model, frame))
            .wrap_err("when rendering terminal")?;
        Ok(())
    }
}

fn draw_frame(model: &Model, frame: &mut Frame) {
    use tui::widgets::*;

    let size = frame.size();
    let chat = model
        .chat_messages
        .iter()
        .map(|msg| ListItem::new(format!("{}: {}", msg.sender_name, msg.message)))
        .collect::<Vec<_>>();
    let chat = List::new(chat)
        .block(Block::default().title("Chat").borders(Borders::all()))
        .highlight_style(Style::default())
        .start_corner(Corner::BottomLeft);
    frame.render_widget(chat, size);
}
