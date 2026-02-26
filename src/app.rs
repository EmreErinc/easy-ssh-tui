use crate::ssh::{get_ssh_keys, SshKey};
use arboard::Clipboard;
use ratatui::widgets::ListState;
use std::time::{Duration, Instant};

pub struct App {
    pub keys: Vec<SshKey>,
    pub list_state: ListState,
    pub clipboard_msg: Option<(String, Instant)>,
    clipboard: Option<Clipboard>,
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
}
