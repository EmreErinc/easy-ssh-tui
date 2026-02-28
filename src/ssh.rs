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
