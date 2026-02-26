# easy-ssh

A simple, fast terminal application built with Rust and [Ratatui](https://github.com/ratatui-org/ratatui) to easily manage and view your SSH keys.

## Features

- **View SSH Keys:** Displays a list of your SSH keys found in the `~/.ssh` directory.
- **Key Details:** Automatically shows the associated public and private key contents for the selected SSH key.
- **Copy Public Key:** Quickly copy the selected public key to your clipboard with a single keystroke.
- **Terminal UI:** Lightweight and responsive terminal interface.

## Requirements

- [Rust](https://www.rust-lang.org/tools/install) (Cargo)
- An existing `~/.ssh` directory with SSH keys.

## Installation

Clone the repository and build the project using Cargo:

```bash
git clone <repository-url>
cd easy-ssh
cargo build --release
```

You can then run the executable located at `./target/release/easy-ssh`, or install it globally:

```bash
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
- <kbd>c</kbd>: Copy the selected public key to the clipboard
- <kbd>q</kbd> or <kbd>Esc</kbd>: Quit the application

## Dependencies

- **[ratatui](https://crates.io/crates/ratatui):** Terminal UI library
- **[crossterm](https://crates.io/crates/crossterm):** Cross-platform terminal manipulation
- **[arboard](https://crates.io/crates/arboard):** Cross-platform clipboard support
- **[dirs](https://crates.io/crates/dirs):** System directory paths
