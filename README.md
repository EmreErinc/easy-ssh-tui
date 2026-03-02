# easy-ssh-tui

A simple, fast terminal application built with Rust and [Ratatui](https://github.com/ratatui-org/ratatui) to easily manage and view your SSH keys.

## Features

- **View SSH Keys:** Displays a list of your SSH keys found in the `~/.ssh` directory.
- **Key Details:** Automatically shows the associated public and private key contents for the selected SSH key.
- **Copy Public Key:** Quickly copy the selected public key to your clipboard with a single keystroke.
- **SSH Key Generation:** Create new `ed25519` SSH keys directly from the UI.
- **PEM File Importer:** Securely import existing `.pem` files from your filesystem with automatic permission handling (`chmod 600`) and passphrase support via `ssh-add`.
- **SSH Config Editor:** View, add, edit, and delete `~/.ssh/config` host entries directly from the TUI.
- **Known Hosts Viewer:** Browse and manage `~/.ssh/known_hosts` entries with the ability to delete stale hosts.
- **Tab-based Navigation:** Switch between Keys, Config, and Known Hosts views using number keys.
- **Terminal UI:** Lightweight and responsive terminal interface.

## Release Notes
Please see the [CHANGELOG.md](CHANGELOG.md) for the latest release notes and history.

## Requirements

- [Rust](https://www.rust-lang.org/tools/install) (Cargo)
- An existing `~/.ssh` directory with SSH keys.

## Installation

Clone the repository and build the project using Cargo:

```bash
git clone https://github.com/yourusername/easy-ssh-tui.git
cd easy-ssh-tui
```

If you just want to install and use it immediately, you can run the provided release script (Mac/Linux):

```bash
./release.sh
```

Alternatively, you can build and install it manually via Cargo:

```bash
cargo build --release
cargo install --path .
```

## Usage

Run the application from your terminal:

```bash
easy-ssh
```

Or run it directly using `cargo` if you are in the project directory:

```bash
cargo run
```

### Keybindings

#### Global
- <kbd>1</kbd>: Switch to **Keys** tab
- <kbd>2</kbd>: Switch to **Config** tab
- <kbd>3</kbd>: Switch to **Known Hosts** tab
- <kbd>Up</kbd> or <kbd>k</kbd>: Move selection up
- <kbd>Down</kbd> or <kbd>j</kbd>: Move selection down
- <kbd>q</kbd> or <kbd>Esc</kbd>: Quit the application

#### Keys Tab
- <kbd>n</kbd>: Create a new SSH key
- <kbd>i</kbd>: Import a `.pem` file
- <kbd>c</kbd>: Copy the selected public key to clipboard

#### Config Tab
- <kbd>a</kbd>: Add a new SSH config entry
- <kbd>e</kbd>: Edit the selected entry
- <kbd>d</kbd>: Delete the selected entry

#### Known Hosts Tab
- <kbd>d</kbd>: Delete the selected known host entry

## Dependencies

- **[ratatui](https://crates.io/crates/ratatui):** Terminal UI library
- **[crossterm](https://crates.io/crates/crossterm):** Cross-platform terminal manipulation
- **[arboard](https://crates.io/crates/arboard):** Cross-platform clipboard support
- **[dirs](https://crates.io/crates/dirs):** System directory paths
