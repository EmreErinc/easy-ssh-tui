pub mod app;
pub mod ssh;
pub mod ui;

use app::{ActiveTab, App, InputMode};
use crossterm::{
    cursor::{Hide, Show},
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
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture, Hide)?;
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
        DisableMouseCapture,
        Show
    )?;

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
                    // Tab switching
                    KeyCode::Char('1') => app.switch_tab(ActiveTab::Keys),
                    KeyCode::Char('2') => app.switch_tab(ActiveTab::SshConfig),
                    KeyCode::Char('3') => app.switch_tab(ActiveTab::KnownHosts),
                    // Navigation (works across all tabs)
                    KeyCode::Down | KeyCode::Char('j') => match app.active_tab {
                        ActiveTab::Keys => app.next(),
                        ActiveTab::SshConfig => app.config_next(),
                        ActiveTab::KnownHosts => app.kh_next(),
                    },
                    KeyCode::Up | KeyCode::Char('k') => match app.active_tab {
                        ActiveTab::Keys => app.previous(),
                        ActiveTab::SshConfig => app.config_previous(),
                        ActiveTab::KnownHosts => app.kh_previous(),
                    },
                    // Tab-specific actions
                    KeyCode::Char('c') if app.active_tab == ActiveTab::Keys => app.copy_public_key(),
                    KeyCode::Char('n') if app.active_tab == ActiveTab::Keys => app.start_creation(),
                    KeyCode::Char('i') if app.active_tab == ActiveTab::Keys => app.start_file_browser(),
                    KeyCode::Char('a') if app.active_tab == ActiveTab::SshConfig => app.start_add_config(),
                    KeyCode::Char('e') if app.active_tab == ActiveTab::SshConfig => app.start_edit_config(),
                    KeyCode::Char('d') if app.active_tab == ActiveTab::SshConfig => app.delete_config_entry(),
                    KeyCode::Char('d') if app.active_tab == ActiveTab::KnownHosts => app.delete_known_host(),
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
                },
                InputMode::ConfigEditing => match key.code {
                    KeyCode::Enter => app.confirm_config_edit(),
                    KeyCode::Char(c) => app.config_edit_input(c),
                    KeyCode::Backspace => app.config_edit_backspace(),
                    KeyCode::Tab => app.config_edit_next_field(),
                    KeyCode::Esc => app.cancel_config_edit(),
                    _ => {}
                },
            }
        }
    }
}
