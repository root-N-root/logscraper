use std::io;

use ratatui::{
    Frame, Terminal,
    crossterm::event::{self, Event},
    prelude::Backend,
};

pub mod app;
pub mod ui;

pub async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut app::App) -> io::Result<bool> {
    loop {
        terminal.draw(|frame| ui::ui(frame, app))?;
        if event::poll(std::time::Duration::from_millis(1000))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Release {
                    continue;
                }
                app.handle(key.code);
                if app.exit_approved {
                    return Ok(true);
                }
            }
        }
        // Update logs from the channel
        app.update_logs();
    }
}
