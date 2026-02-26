use crate::ssh::{create_ssh_key, get_ssh_keys, SshKey};
use arboard::Clipboard;
use ratatui::widgets::ListState;
use std::time::{Duration, Instant};

#[derive(PartialEq)]
pub enum InputMode {
    Normal,
    Editing,
}

#[derive(PartialEq)]
pub enum InputField {
    Name,
    Email,
}

pub struct App {
    pub keys: Vec<SshKey>,
    pub list_state: ListState,
    pub clipboard_msg: Option<(String, Instant)>,
    clipboard: Option<Clipboard>,
    
    // Key Creation State
    pub input_mode: InputMode,
    pub input_field: InputField,
    pub input_name: String,
    pub input_email: String,
    pub popup_msg: Option<String>,
}

impl App {
    pub fn new() -> App {
        let keys = get_ssh_keys();
        let mut list_state = ListState::default();
        if !keys.is_empty() {
            list_state.select(Some(0));
        }

        let clipboard = Clipboard::new().ok();

        App {
            keys,
            list_state,
            clipboard_msg: None,
            clipboard,
            input_mode: InputMode::Normal,
            input_field: InputField::Name,
            input_name: String::new(),
            input_email: String::new(),
            popup_msg: None,
        }
    }

    pub fn next(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.keys.len().saturating_sub(1) {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.keys.len().saturating_sub(1)
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn copy_public_key(&mut self) {
        if let Some(selected) = self.list_state.selected() {
            if let Some(key) = self.keys.get(selected) {
                if let Some(cb) = &mut self.clipboard {
                    if cb.set_text(key.public_content.clone()).is_ok() {
                        self.clipboard_msg = Some((
                            format!("Copied {} public key to clipboard!", key.name),
                            Instant::now(),
                        ));
                    } else {
                        self.clipboard_msg = Some((
                            "Failed to copy to clipboard".to_string(),
                            Instant::now(),
                        ));
                    }
                } else {
                    self.clipboard_msg = Some((
                        "Clipboard not available".to_string(),
                        Instant::now(),
                    ));
                }
            }
        }
    }

    pub fn update_clipboard_msg_timeout(&mut self) {
        if let Some((_, time)) = self.clipboard_msg {
            if time.elapsed() > Duration::from_secs(3) {
                self.clipboard_msg = None;
            }
        }
    }

    pub fn start_creation(&mut self) {
        self.input_mode = InputMode::Editing;
        self.input_field = InputField::Name;
        self.input_name.clear();
        self.input_email.clear();
        self.popup_msg = None;
    }

    pub fn cancel_creation(&mut self) {
        self.input_mode = InputMode::Normal;
        self.popup_msg = None;
    }

    pub fn handle_input(&mut self, c: char) {
        match self.input_field {
            InputField::Name => {
                if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                    self.input_name.push(c);
                }
            }
            InputField::Email => {
                self.input_email.push(c);
            }
        }
    }

    pub fn handle_backspace(&mut self) {
        match self.input_field {
            InputField::Name => {
                self.input_name.pop();
            }
            InputField::Email => {
                self.input_email.pop();
            }
        }
    }

    pub fn switch_field(&mut self) {
        self.input_field = match self.input_field {
            InputField::Name => InputField::Email,
            InputField::Email => InputField::Name,
        };
    }

    pub fn confirm_creation(&mut self) {
        if self.input_name.is_empty() {
            self.popup_msg = Some("Key name cannot be empty".to_string());
            return;
        }

        match create_ssh_key(&self.input_name, &self.input_email) {
            Ok(_) => {
                self.keys = get_ssh_keys(); // refresh keys
                if let Some(pos) = self.keys.iter().position(|k| k.name == self.input_name) {
                    self.list_state.select(Some(pos));
                }
                self.input_mode = InputMode::Normal;
                self.clipboard_msg = Some((
                    format!("Created SSH key '{}' successfully!", self.input_name),
                    Instant::now(),
                ));
            }
            Err(e) => {
                self.popup_msg = Some(format!("Error: {}", e));
            }
        }
    }
}
