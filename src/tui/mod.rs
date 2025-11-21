mod app;
mod ui;
mod vitals;
mod disassembly;
mod probes;
mod logs;
mod input;

use crate::config::Config;
use crate::error::Result;
use app::App;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;

pub async fn run_interactive(config: Config) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new(config);

    // Run the app
    let res = run_app(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Error: {:?}", err);
    }

    Ok(())
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        // Handle input with timeout
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        return Ok(());
                    }
                    KeyCode::Char('q') if !app.input_mode => {
                        return Ok(());
                    }
                    KeyCode::Esc => {
                        app.input_mode = false;
                    }
                    KeyCode::Char('i') if !app.input_mode => {
                        app.input_mode = true;
                    }
                    KeyCode::Enter if app.input_mode => {
                        let command = app.input.clone();
                        app.input.clear();
                        app.input_mode = false;
                        app.handle_command(command).await?;
                    }
                    KeyCode::Char(c) if app.input_mode => {
                        app.input.push(c);
                    }
                    KeyCode::Backspace if app.input_mode => {
                        app.input.pop();
                    }
                    KeyCode::Up if !app.input_mode => {
                        app.scroll_logs_up();
                    }
                    KeyCode::Down if !app.input_mode => {
                        app.scroll_logs_down();
                    }
                    KeyCode::Tab => {
                        app.next_focus();
                    }
                    _ => {}
                }
            }
        }

        // Update vitals periodically
        app.update_vitals();
    }
}
