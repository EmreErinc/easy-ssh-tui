pub mod app;
pub mod ssh;
pub mod ui;

use app::{App, InputMode};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{error::Error, io};

fn main() -> Result<(), Box<dyn Error>> {
    // Check if ssh directory exists
    let ssh_dir = dirs::home_dir().map(|mut p| {
        p.push(".ssh");
        p
    });

    if ssh_dir.is_none() || !ssh_dir.unwrap().exists() {
        println!("No ~/.ssh directory found.");
        return Ok(());
    }

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

// ... existing code ...

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<(), Box<dyn Error>> 
where
    <B as ratatui::backend::Backend>::Error: std::convert::Into<Box<dyn std::error::Error>> + 'static,
{
    loop {
        terminal.draw(|f| ui::ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            match app.input_mode {
                InputMode::Normal => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Down | KeyCode::Char('j') => app.next(),
                    KeyCode::Up | KeyCode::Char('k') => app.previous(),
                    KeyCode::Char('c') => app.copy_public_key(),
                    KeyCode::Char('n') => app.start_creation(),
                    KeyCode::Char('i') => app.start_file_browser(),
                    _ => {}
                },
                InputMode::Editing => match key.code {
                    KeyCode::Enter => app.confirm_creation(),
                    KeyCode::Char(c) => app.handle_input(c),
                    KeyCode::Backspace => app.handle_backspace(),
                    KeyCode::Esc => app.cancel_creation(),
                    KeyCode::Tab => app.switch_field(),
                    _ => {}
                },
                InputMode::FileBrowser => match key.code {
                    KeyCode::Down | KeyCode::Char('j') => app.fb_next(),
                    KeyCode::Up | KeyCode::Char('k') => app.fb_previous(),
                    KeyCode::Enter => app.fb_select(),
                    KeyCode::Left | KeyCode::Backspace => app.fb_parent(),
                    KeyCode::Esc | KeyCode::Char('q') => app.cancel_import(),
                    _ => {}
                },
                InputMode::ImportAction => match key.code {
                    KeyCode::Char('m') => app.handle_import_action('m'),
                    KeyCode::Char('c') => app.handle_import_action('c'),
                    KeyCode::Esc | KeyCode::Char('q') => app.cancel_import(),
                    _ => {}
                },
                InputMode::PasswordPrompt => match key.code {
                    KeyCode::Enter => app.submit_password(),
                    KeyCode::Char(c) => app.handle_password_input(c),
                    KeyCode::Backspace => app.handle_password_backspace(),
                    KeyCode::Esc => app.cancel_import(),
                    _ => {}
                }
            }
        }
    }
}
