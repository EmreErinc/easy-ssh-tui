use crate::ssh::{create_ssh_key, get_ssh_keys, SshKey};
use arboard::Clipboard;
use ratatui::widgets::ListState;
use std::time::{Duration, Instant};

#[derive(PartialEq)]
pub enum InputMode {
    Normal,
    Editing,
    FileBrowser,
    ImportAction,
    PasswordPrompt,
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

    // File Browser State
    pub current_dir: std::path::PathBuf,
    pub file_entries: Vec<std::path::PathBuf>,
    pub file_list_state: ListState,
    
    // Import State
    pub selected_import_file: Option<std::path::PathBuf>,
    pub password_input: String,
}

impl App {
    pub fn new() -> App {
        let keys = get_ssh_keys();
        let mut list_state = ListState::default();
        if !keys.is_empty() {
            list_state.select(Some(0));
        }

        let clipboard = Clipboard::new().ok();
        
        // Initialize file browser to home directory
        let home_dir = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/"));
        
        let mut app = App {
            keys,
            list_state,
            clipboard_msg: None,
            clipboard,
            input_mode: InputMode::Normal,
            input_field: InputField::Name,
            input_name: String::new(),
            input_email: String::new(),
            popup_msg: None,
            current_dir: home_dir,
            file_entries: Vec::new(),
            file_list_state: ListState::default(),
            selected_import_file: None,
            password_input: String::new(),
        };

        app.load_directory();
        app
    }

    pub fn load_directory(&mut self) {
        self.file_entries.clear();
        if let Ok(entries) = std::fs::read_dir(&self.current_dir) {
            let mut dirs = Vec::new();
            let mut files = Vec::new();
            
            for entry in entries.flatten() {
                let path = entry.path();
                // Optionally filter to only show directories and .pem / .pub files to reduce noise
                // But a general browser is okay too. Let's just show all for now.
                if path.is_dir() {
                    dirs.push(path);
                } else {
                    files.push(path);
                }
            }
            
            // Sort alphabetically
            dirs.sort();
            files.sort();
            
            if let Some(parent) = self.current_dir.parent() {
                self.file_entries.push(parent.to_path_buf());
            }
            self.file_entries.extend(dirs);
            self.file_entries.extend(files);
        }
        
        if !self.file_entries.is_empty() {
            self.file_list_state.select(Some(0));
        } else {
            self.file_list_state.select(None);
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

    // --- File Browser Methods ---

    pub fn start_file_browser(&mut self) {
        self.input_mode = InputMode::FileBrowser;
        self.popup_msg = None;
        self.load_directory();
    }

    pub fn fb_next(&mut self) {
        let i = match self.file_list_state.selected() {
            Some(i) => {
                if i >= self.file_entries.len().saturating_sub(1) {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.file_list_state.select(Some(i));
    }

    pub fn fb_previous(&mut self) {
        let i = match self.file_list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.file_entries.len().saturating_sub(1)
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.file_list_state.select(Some(i));
    }

    pub fn fb_select(&mut self) {
        if let Some(selected) = self.file_list_state.selected() {
            if let Some(path) = self.file_entries.get(selected) {
                if path.is_dir() {
                    self.current_dir = path.clone();
                    self.load_directory();
                } else {
                    self.selected_import_file = Some(path.clone());
                    self.input_mode = InputMode::ImportAction;
                }
            }
        }
    }

    pub fn fb_parent(&mut self) {
        if let Some(parent) = self.current_dir.parent() {
            self.current_dir = parent.to_path_buf();
            self.load_directory();
        }
    }

    pub fn cancel_import(&mut self) {
        self.input_mode = InputMode::Normal;
        self.selected_import_file = None;
        self.password_input.clear();
        self.popup_msg = None;
    }

    pub fn handle_import_action(&mut self, action_char: char) {
        let is_move = match action_char {
            'm' => true,
            'c' => false,
            _ => return, // Ignore other keys
        };

        if let Some(path) = &self.selected_import_file {
            match crate::ssh::handle_pem_import(path, is_move) {
                Ok(new_path) => {
                    if crate::ssh::needs_password(&new_path) {
                        self.input_mode = InputMode::PasswordPrompt;
                        self.selected_import_file = Some(new_path); // Update to new path in ~/.ssh
                        self.password_input.clear();
                        self.popup_msg = None;
                    } else {
                        // Import successful without password
                        self.keys = get_ssh_keys();
                        if let Some(pos) = self.keys.iter().position(|k| k.private_path == new_path) {
                            self.list_state.select(Some(pos));
                        }
                        self.cancel_import();
                        self.clipboard_msg = Some(("Import successful!".to_string(), Instant::now()));
                    }
                }
                Err(e) => {
                    self.popup_msg = Some(format!("Import error: {}", e));
                }
            }
        }
    }

    pub fn handle_password_input(&mut self, c: char) {
        self.password_input.push(c);
    }

    pub fn handle_password_backspace(&mut self) {
        self.password_input.pop();
    }

    pub fn submit_password(&mut self) {
        if let Some(path) = &self.selected_import_file {
            match crate::ssh::ssh_add_with_password(path, &self.password_input) {
                Ok(_) => {
                    self.keys = get_ssh_keys();
                    if let Some(pos) = self.keys.iter().position(|k| k.private_path == *path) {
                        self.list_state.select(Some(pos));
                    }
                    self.cancel_import();
                    self.clipboard_msg = Some(("Import and ssh-add successful!".to_string(), Instant::now()));
                }
                Err(e) => {
                    self.popup_msg = Some(format!("ssh-add error: {}", e));
                    self.password_input.clear();
                }
            }
        }
    }
}
