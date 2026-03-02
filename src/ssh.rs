use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub fn create_ssh_key(name: &str, email: &str) -> std::io::Result<()> {
    let mut ssh_dir = dirs::home_dir().ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Home directory not found"))?;
    ssh_dir.push(".ssh");
    
    let key_path = ssh_dir.join(name);
    let key_path_str = key_path.to_str().ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid path"))?;

    // 1. Run ssh-keygen
    let status = Command::new("ssh-keygen")
        .args(["-t", "ed25519", "-C", email, "-f", key_path_str, "-N", ""])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;

    if !status.success() {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "ssh-keygen failed"));
    }

    // 2. Try to run ssh-add
    // Note: This might fail if the ssh-agent is not running in the current context, 
    // but we generally just ignore the failure and let the user add it themselves if needed,
    // or we can just run it and not strictly error out if it fails.
    let _ = Command::new("ssh-add")
        .arg(key_path_str)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    Ok(())
}

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

pub fn handle_pem_import(path: &Path, move_file: bool) -> std::io::Result<PathBuf> {
    let mut ssh_dir = dirs::home_dir().ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Home directory not found"))?;
    ssh_dir.push(".ssh");

    let file_name = path.file_name().ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid file name"))?;
    let new_path = ssh_dir.join(file_name);

    if move_file {
        fs::rename(path, &new_path)?;
    } else {
        fs::copy(path, &new_path)?;
    }

    // chmod 600
    let mut perms = fs::metadata(&new_path)?.permissions();
    perms.set_mode(0o600);
    fs::set_permissions(&new_path, perms)?;

    Ok(new_path)
}

pub fn needs_password(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    // Use ssh-keygen to check if it has a passphrase. 
    // -y reads the private key and prints the public key. 
    // -P "" tries to use an empty passphrase. 
    // If it fails, it usually means it's encrypted.
    let status = Command::new("ssh-keygen")
        .args(["-y", "-P", "", "-f", &path_str])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    match status {
        Ok(s) => !s.success(), // If it's not successful, it probably needs a password
        Err(_) => true, // Assume it needs one if command fails entirely
    }
}

pub fn ssh_add_with_password(path: &Path, password: &str) -> std::io::Result<()> {
    let path_str = path.to_string_lossy();
    let script_path = "/tmp/easy-ssh-askpass.sh";

    // Write a temporary shell script that acts as SSH_ASKPASS
    let script_content = format!("#!/bin/sh\necho \"{}\"\n", password);
    fs::write(script_path, script_content)?;
    
    // Make it executable
    let mut perms = fs::metadata(script_path)?.permissions();
    perms.set_mode(0o700);
    fs::set_permissions(script_path, perms)?;

    let status = Command::new("ssh-add")
        .arg(&*path_str) // deref Cow to &str
        .env("SSH_ASKPASS_REQUIRE", "force")
        .env("SSH_ASKPASS", script_path)
        .env("DISPLAY", "dummy:0")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    // Clean up script
    let _ = fs::remove_file(script_path);

    match status {
        Ok(s) if s.success() => Ok(()),
        _ => Err(std::io::Error::new(std::io::ErrorKind::Other, "Failed to add key or incorrect password")),
    }
}

// --- SSH Config ---

#[derive(Clone)]
pub struct SshConfigEntry {
    pub host: String,
    pub hostname: String,
    pub user: String,
    pub port: String,
    pub identity_file: String,
}

impl SshConfigEntry {
    pub fn new() -> Self {
        Self {
            host: String::new(),
            hostname: String::new(),
            user: String::new(),
            port: String::from("22"),
            identity_file: String::new(),
        }
    }
}

pub fn parse_ssh_config() -> Vec<SshConfigEntry> {
    let mut entries = Vec::new();
    let mut ssh_dir = match dirs::home_dir() {
        Some(d) => d,
        None => return entries,
    };
    ssh_dir.push(".ssh");
    ssh_dir.push("config");

    let content = match fs::read_to_string(&ssh_dir) {
        Ok(c) => c,
        Err(_) => return entries,
    };

    let mut current: Option<SshConfigEntry> = None;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if let Some((key, value)) = trimmed.split_once(char::is_whitespace) {
            let key_lower = key.to_lowercase();
            let value = value.trim().to_string();

            match key_lower.as_str() {
                "host" => {
                    if let Some(entry) = current.take() {
                        entries.push(entry);
                    }
                    let mut entry = SshConfigEntry::new();
                    entry.host = value;
                    current = Some(entry);
                }
                "hostname" => {
                    if let Some(ref mut entry) = current {
                        entry.hostname = value;
                    }
                }
                "user" => {
                    if let Some(ref mut entry) = current {
                        entry.user = value;
                    }
                }
                "port" => {
                    if let Some(ref mut entry) = current {
                        entry.port = value;
                    }
                }
                "identityfile" => {
                    if let Some(ref mut entry) = current {
                        entry.identity_file = value;
                    }
                }
                _ => {}
            }
        }
    }

    if let Some(entry) = current.take() {
        entries.push(entry);
    }

    entries
}

pub fn save_ssh_config(entries: &[SshConfigEntry]) -> std::io::Result<()> {
    let mut ssh_path = dirs::home_dir()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Home not found"))?;
    ssh_path.push(".ssh");
    ssh_path.push("config");

    let mut content = String::new();
    for entry in entries {
        content.push_str(&format!("Host {}\n", entry.host));
        if !entry.hostname.is_empty() {
            content.push_str(&format!("    HostName {}\n", entry.hostname));
        }
        if !entry.user.is_empty() {
            content.push_str(&format!("    User {}\n", entry.user));
        }
        if !entry.port.is_empty() && entry.port != "22" {
            content.push_str(&format!("    Port {}\n", entry.port));
        }
        if !entry.identity_file.is_empty() {
            content.push_str(&format!("    IdentityFile {}\n", entry.identity_file));
        }
        content.push('\n');
    }

    fs::write(&ssh_path, content)
}

pub fn add_ssh_config_entry(entry: &SshConfigEntry) -> std::io::Result<()> {
    let mut entries = parse_ssh_config();
    entries.push(entry.clone());
    save_ssh_config(&entries)
}

pub fn remove_ssh_config_entry(index: usize) -> std::io::Result<()> {
    let mut entries = parse_ssh_config();
    if index < entries.len() {
        entries.remove(index);
        save_ssh_config(&entries)?;
    }
    Ok(())
}

// --- Known Hosts ---

#[derive(Clone)]
pub struct KnownHostEntry {
    pub hostname: String,
    pub key_type: String,
    pub fingerprint: String,
}

pub fn parse_known_hosts() -> Vec<KnownHostEntry> {
    let mut entries = Vec::new();
    let mut path = match dirs::home_dir() {
        Some(d) => d,
        None => return entries,
    };
    path.push(".ssh");
    path.push("known_hosts");

    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return entries,
    };

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = trimmed.splitn(3, ' ').collect();
        if parts.len() >= 2 {
            entries.push(KnownHostEntry {
                hostname: parts[0].to_string(),
                key_type: parts[1].to_string(),
                fingerprint: if parts.len() > 2 {
                    // Show only the first 30 chars of the key for display
                    let fp = parts[2];
                    if fp.len() > 30 {
                        format!("{}...", &fp[..30])
                    } else {
                        fp.to_string()
                    }
                } else {
                    String::new()
                },
            });
        }
    }

    entries
}

pub fn delete_known_host(index: usize) -> std::io::Result<()> {
    let mut path = dirs::home_dir()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Home not found"))?;
    path.push(".ssh");
    path.push("known_hosts");

    let content = fs::read_to_string(&path)?;
    let lines: Vec<&str> = content.lines().collect();

    // Filter out blank/comment lines to map display index to real line index
    let mut real_indices: Vec<usize> = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if !trimmed.is_empty() && !trimmed.starts_with('#') {
            real_indices.push(i);
        }
    }

    if index >= real_indices.len() {
        return Ok(());
    }

    let line_to_remove = real_indices[index];
    let new_content: String = lines
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != line_to_remove)
        .map(|(_, line)| *line)
        .collect::<Vec<&str>>()
        .join("\n");

    fs::write(&path, new_content + "\n")
}

// --- Export to GitHub/GitLab ---

pub fn export_key_to_github(token: &str, title: &str, key: &str) -> Result<String, String> {
    let body = format!(
        r#"{{"title":"{}","key":"{}"}}"#,
        title.replace('"', r#"\""#),
        key.trim().replace('"', r#"\""#)
    );

    let response = ureq::post("https://api.github.com/user/keys")
        .header("Authorization", &format!("Bearer {}", token))
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "easy-ssh-tui")
        .header("Content-Type", "application/json")
        .send(body.as_bytes());

    match response {
        Ok(resp) => {
            let status = resp.status();
            if status == 201 {
                Ok("Key successfully added to GitHub!".to_string())
            } else {
                let body_text = resp.into_body().read_to_string().unwrap_or_default();
                Err(format!("GitHub API error ({}): {}", status, body_text))
            }
        }
        Err(e) => Err(format!("Request failed: {}", e)),
    }
}

pub fn export_key_to_gitlab(token: &str, title: &str, key: &str) -> Result<String, String> {
    let body = format!(
        r#"{{"title":"{}","key":"{}"}}"#,
        title.replace('"', r#"\""#),
        key.trim().replace('"', r#"\""#)
    );

    let response = ureq::post("https://gitlab.com/api/v4/user/keys")
        .header("PRIVATE-TOKEN", token)
        .header("Content-Type", "application/json")
        .send(body.as_bytes());

    match response {
        Ok(resp) => {
            let status = resp.status();
            if status == 201 {
                Ok("Key successfully added to GitLab!".to_string())
            } else {
                let body_text = resp.into_body().read_to_string().unwrap_or_default();
                Err(format!("GitLab API error ({}): {}", status, body_text))
            }
        }
        Err(e) => Err(format!("Request failed: {}", e)),
    }
}
