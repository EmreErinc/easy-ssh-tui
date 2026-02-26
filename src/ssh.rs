use std::fs;
use std::path::PathBuf;

pub struct SshKey {
    pub name: String,
    pub private_path: PathBuf,
    pub public_path: PathBuf,
    pub private_content: String,
    pub public_content: String,
}

impl SshKey {
    pub fn new(private_path: PathBuf, public_path: PathBuf) -> Option<Self> {
        let name = private_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();

        let private_content = fs::read_to_string(&private_path).unwrap_or_default();
        let public_content = fs::read_to_string(&public_path).unwrap_or_default();

        if private_content.is_empty() && public_content.is_empty() {
            return None;
        }

        Some(Self {
            name,
            private_path,
            public_path,
            private_content,
            public_content,
        })
    }
}

pub fn get_ssh_keys() -> Vec<SshKey> {
    let mut keys = Vec::new();

    let mut ssh_dir = match dirs::home_dir() {
        Some(dir) => dir,
        None => return keys,
    };
    ssh_dir.push(".ssh");

    if !ssh_dir.exists() || !ssh_dir.is_dir() {
        return keys;
    }

    if let Ok(entries) = fs::read_dir(ssh_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                // Look for .pub files
                if let Some(extension) = path.extension() {
                    if extension == "pub" {
                        let public_path = path.clone();
                        let private_path = path.with_extension("");

                        if private_path.exists() {
                            if let Some(key) = SshKey::new(private_path, public_path) {
                                keys.push(key);
                            }
                        }
                    }
                }
            }
        }
    }

    // Sort keys by name
    keys.sort_by(|a, b| a.name.cmp(&b.name));
    keys
}
