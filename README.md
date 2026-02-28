# easy-ssh

A simple, fast terminal application built with Rust and [Ratatui](https://github.com/ratatui-org/ratatui) to easily manage and view your SSH keys.

## Features

- **View SSH Keys:** Displays a list of your SSH keys found in the `~/.ssh` directory.
- **Key Details:** Automatically shows the associated public and private key contents for the selected SSH key.
- **Copy Public Key:** Quickly copy the selected public key to your clipboard with a single keystroke.
- **SSH Key Generation:** Create new `ed25519` SSH keys directly from the UI.
- **PEM File Importer:** Securely import existing `.pem` files from your filesystem with automatic permission handling (`chmod 600`) and passphrase support via `ssh-add`.
- **Terminal UI:** Lightweight and responsive terminal interface.

## Release Notes
Please see the [CHANGELOG.md](CHANGELOG.md) for the latest release notes and history.

## Requirements

- [Rust](https://www.rust-lang.org/tools/install) (Cargo)
- An existing `~/.ssh` directory with SSH keys.

## Installation

Clone the repository and build the project using Cargo:

```bash
git clone https://github.com/yourusername/easy-ssh.git
cd easy-ssh
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

- <kbd>Up</kbd> or <kbd>k</kbd>: Move selection up
- <kbd>Down</kbd> or <kbd>j</kbd>: Move selection down
- <kbd>n</kbd>: Open the prompt to **create** a new SSH key
- <kbd>i</kbd>: Open the file browser to **import** an existing `.pem` file
- <kbd>c</kbd>: Copy the selected public key to the clipboard
- <kbd>q</kbd> or <kbd>Esc</kbd>: Quit the application

## Dependencies

- **[ratatui](https://crates.io/crates/ratatui):** Terminal UI library
- **[crossterm](https://crates.io/crates/crossterm):** Cross-platform terminal manipulation
- **[arboard](https://crates.io/crates/arboard):** Cross-platform clipboard support
- **[dirs](https://crates.io/crates/dirs):** System directory paths
