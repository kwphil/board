use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders},
    Terminal,
};

use crossterm::{
    execute,
    terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    event::{self, Event, KeyCode, KeyModifiers},
};

use std::{
    io,
    time::Duration,
};

fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?; 

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.clear()?;

    loop {
        // Checking for a SIGINT for now, have something more graceful later
        if event::poll(Duration::from_millis(0))? {
            if let Event::Key(key_event) = event::read()? {
                if key_event.code == KeyCode::Char('c')
                    && key_event.modifiers.contains(KeyModifiers::CONTROL)
                {
                    disable_raw_mode()?;
                    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
                    terminal.show_cursor()?;
                    break Ok(());
                }
            }
        }

        terminal.draw(|f| {
            let size = f.area();

            let vertical = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(50),
                    Constraint::Percentage(50),
                ])
                .split(size);

            let top_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(50),
                    Constraint::Percentage(50),
                ])
                .split(vertical[0]);

            let top_left = Block::default().title("Diagnostics (d)").borders(Borders::ALL);
            let top_right = Block::default().title("Filesystem (f)").borders(Borders::ALL);
            let bottom = Block::default().title("Shell (s)").borders(Borders::ALL);

            f.render_widget(top_left, top_chunks[0]);
            f.render_widget(top_right, top_chunks[1]);
            f.render_widget(bottom, vertical[1]);
        })?;
    }
}
