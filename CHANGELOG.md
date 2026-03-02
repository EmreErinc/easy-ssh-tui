# Changelog

All notable changes to this project will be documented in this file.

## [0.1.2] - 2026-03-02

### Added
- **SSH Config Editor:** View, add, edit, and delete `~/.ssh/config` host entries (Host, HostName, User, Port, IdentityFile) directly from the TUI.
- **Known Hosts Viewer:** Browse and manage `~/.ssh/known_hosts` entries with hostname, key type, and fingerprint display. Delete stale entries with <kbd>d</kbd>.
- **Tab-based Navigation:** Switch between Keys (<kbd>1</kbd>), Config (<kbd>2</kbd>), and Known Hosts (<kbd>3</kbd>) views. The header bar now shows context-sensitive keybinding hints for the active tab.

## [0.1.0] - 2026-02-28

### Added
- **Interactive TUI**: A fast, responsive terminal user interface built with Ratatui and Crossterm.
- **SSH Key Viewer**: Automatically discovers and lists existing SSH keys from the `~/.ssh` directory.
- **Key Inspection**: Displays the contents of both private and public keys for the currently selected key.
- **Clipboard Support**: Copy public keys directly to your system clipboard with the <kbd>c</kbd> keybinding.
- **SSH Key Generation**: Create new `ed25519` SSH keys directly from the UI by pressing <kbd>n</kbd>.
- **PEM File Importer**: Select and securely import existing `.pem` files from your filesystem by pressing <kbd>i</kbd>.
  - Features file browser navigation.
  - Supports both **Move (m)** and **Copy (c)** operations.
  - Automatically handles `chmod 600` permissions.
  - Seamlessly prompts for passphrases for encrypted keys and injects them into `ssh-add` using a temporary `SSH_ASKPASS` script.
- **Release Script**: Added `release.sh` to easily compile the release build and install the application globally via `cargo`.
