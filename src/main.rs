use portable_pty::{
    CommandBuilder, PtySize, native_pty_system
};

use ratatui::{
    Terminal, backend::CrosstermBackend, layout::{Constraint, Direction, Layout}, widgets::{Block, Borders, Paragraph, Wrap}
};

use crossterm::{
    execute,
    terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    event::{self, Event, KeyCode, KeyModifiers},
};
use vt100::Parser;

use std::{
    error::Error, io, sync::{Arc, Mutex}, time::Duration,
};

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?; 

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.clear()?;

    let pty_sys = native_pty_system();
    let pair = pty_sys.openpty(PtySize { 
        rows: 24, 
        cols: 80, 
        pixel_width: 0, 
        pixel_height: 0 
    }).unwrap();

    let term_width = 80;
    let term_height = 24;

    let mut cmd = CommandBuilder::new("bash");
    let mut child = pair.slave.spawn_command(cmd)?;

    let mut reader = pair.master.try_clone_reader()?;
    let mut writer = pair.master.take_writer()?;

    let sh_output = Arc::new(Mutex::new(Parser::new(term_height, term_width, 0)));
    let sh_output_c = sh_output.clone();

    std::thread::spawn(move || {
        let mut buf = [0u8; 1024];
        loop {
           if let Ok(n) = reader.read(&mut buf) {
               if n > 0 {
                   sh_output_c.lock().unwrap().process(&buf[..n]);
               }
           }
        }
    });

    loop {
        if event::poll(Duration::from_millis(0))? 
            && let Event::Key(key_event) = event::read()? 
        {
            // Commands
            if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                // SIGINT
                if key_event.code == KeyCode::Char('c') {
                    disable_raw_mode()?;
                    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
                    terminal.show_cursor()?;
                    break Ok(());
                }
            }

            match key_event.code {
                KeyCode::Char(c) => {
                    writer.write_all(&[c as u8])?;
                }
                KeyCode::Enter => {
                    writer.write_all(b"\n")?;
                }
                KeyCode::Backspace => {
                    writer.write_all(&[0x7f])?;
                }
                // Ignore unknown keys
                _ => {}
            }
        }

        let parser = sh_output.lock().unwrap();
        let screen = parser.screen();
        let mut text = String::new();

        for line in screen.rows(0, term_width) {
            text.push_str(&line);
            text.push('\n');
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
            let bottom = Paragraph::new(text)
                .block(Block::default().title("Shell(s)").borders(Borders::ALL))
                .wrap(Wrap { trim: false });

            f.render_widget(top_left, top_chunks[0]);
            f.render_widget(top_right, top_chunks[1]);
            f.render_widget(bottom, vertical[1]);
        })?;
    }
}
