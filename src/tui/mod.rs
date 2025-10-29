use std::io;

use ratatui::{
    Frame, Terminal,
    crossterm::event::{self, Event},
    prelude::Backend,
};

pub mod app;
pub mod ui;

pub fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut app::App) -> io::Result<bool> {
    loop {
        terminal.draw(|frame| ui::ui(frame, app))?;
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Release {
                continue;
            }
            app.handle(key.code);
        }
    }
}
