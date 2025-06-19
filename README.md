# Proton Prefix Manager

Proton Prefix Manager is a tool for locating and exploring the Proton prefixes created by Steam on Linux. It offers both a command line interface and a simple graphical interface built with [egui](https://github.com/emilk/egui).

## Overview

Steam uses Proton prefixes (Wine environments) to run Windows games on Linux. This project helps you discover where those prefixes are stored so you can inspect or manage them. You can search your installed games, locate the prefix for a specific game, and open it in your file manager. When run without any arguments, the application launches a GUI that lists your games and shows prefix details.

When multiple Steam users exist, Proton Prefix Manager checks `loginusers.vdf` under Steam's `config` directory and uses the account marked with `"MostRecent" "1"`. If that file is missing, the tool falls back to `~/.steam/config/loginusers.vdf` and `~/.steam/root/config/loginusers.vdf`. `loginusers.vdf` stores 64-bit SteamIDs, which the tool converts to their 32-bit account IDs when resolving the corresponding `localconfig.vdf`. Launch options are read from and written to that user's `localconfig.vdf`.

## Installation

1. Install [Rust](https://www.rust-lang.org/tools/install) and `cargo`.
2. Clone the repository and build:
   ```bash
   git clone https://github.com/D1G1T4L3CH0/proton-prefix-manager.git
   cd proton-prefix-manager
   cargo build --release
   ```
   The resulting binary will be located at `target/release/proton-prefix-manager`.

Alternatively, you can install directly from the source using:

```bash
cargo install --path .
```

## Usage

### Graphical interface

Run the program without arguments to launch the GUI:

```bash
proton-prefix-manager
```

The GUI lists your installed Steam games and shows details about each prefix. You can copy or open the prefix path, run Protontricks or launch `winecfg` for the selected game, and follow external links such as SteamDB or ProtonDB. Game settings let you choose from built‑in and custom Proton versions discovered in `compatibilitytools.d`.

### Command line interface

Search for games by name:

```bash
proton-prefix-manager search "portal"
```

Find the prefix path for a specific AppID:

```bash
proton-prefix-manager prefix 620
```

Open a prefix in your file manager:

```bash
proton-prefix-manager open 620
```

Back up a prefix (stored in `~/.local/share/proton-prefix-manager/backups`):

```bash
proton-prefix-manager backup 620
```

Restore a prefix from a backup directory:

```bash
proton-prefix-manager restore 620 /path/to/backup
```

List backups for a game:

```bash
proton-prefix-manager list-backups 620
```

Delete a backup:

```bash
proton-prefix-manager delete-backup /path/to/backup
```

Reset a prefix:

```bash
proton-prefix-manager reset 620
```
**Warning:** Resetting a prefix will permanently delete it. It's prudent to create a backup of your important data or configuration files before performing this or any other critical action so you can restore your system if something goes wrong.

Clear shader cache:

```bash
proton-prefix-manager clear-cache 620
```

Run protontricks:

```bash
proton-prefix-manager protontricks 620
```

Launch winecfg:

```bash
proton-prefix-manager winecfg 620
```

Show detected `localconfig.vdf` paths:

```bash
proton-prefix-manager config-paths
```

The CLI supports JSON (`--json`), plain text (`--plain`), and custom-delimited output using `--delimiter`.

Use `--debug` to print detailed information about paths and files the application interacts with. This enables verbose logging without having to set the `RUST_LOG` environment variable.

## Debug logging

You can still set `RUST_LOG=debug` for low level logging from dependencies, but in most cases the `--debug` flag is sufficient.

## Project goals

- Provide an easy way to locate Proton prefixes for troubleshooting or modding
- Offer both CLI and GUI workflows in a single binary
- Remain lightweight with minimal dependencies
- Serve as a foundation for more advanced prefix management features in the future

This project is released under the MIT license. Contributions are welcome!

