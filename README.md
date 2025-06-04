# Proton Prefix Finder

Proton Prefix Finder is a tool for locating and exploring the Proton prefixes created by Steam on Linux. It offers both a command line interface and a simple graphical interface built with [egui](https://github.com/emilk/egui).

## Overview

Steam uses Proton prefixes (Wine environments) to run Windows games on Linux. This project helps you discover where those prefixes are stored so you can inspect or manage them. You can search your installed games, locate the prefix for a specific game, and open it in your file manager. When run without any arguments, the application launches a GUI that lists your games and shows prefix details.

## Installation

1. Install [Rust](https://www.rust-lang.org/tools/install) and `cargo`.
2. Clone the repository and build:
   ```bash
   git clone https://github.com/D1G1T4L3CH0/proton-prefix-manager.git
   cd proton-prefix-manager
   cargo build --release
   ```
   The resulting binary will be located at `target/release/proton-prefix-finder`.

Alternatively, you can install directly from the source using:

```bash
cargo install --path .
```

## Usage

### Graphical interface

Run the program without arguments to launch the GUI:

```bash
proton-prefix-finder
```

The GUI lists your installed Steam games and shows details about each prefix. You can copy or open the prefix path and follow external links such as SteamDB or ProtonDB.

### Command line interface

Search for games by name:

```bash
proton-prefix-finder search "portal"
```

Find the prefix path for a specific AppID:

```bash
proton-prefix-finder prefix 620
```

Open a prefix in your file manager:

```bash
proton-prefix-finder open 620
```

Back up a prefix to a directory:

```bash
proton-prefix-finder backup 620 /path/to/backup
```

Restore a prefix from a backup:

```bash
proton-prefix-finder restore 620 /path/to/backup
```

The CLI supports JSON (`--json`), plain text (`--plain`), and custom-delimited output using `--delimiter`.

## Project goals

- Provide an easy way to locate Proton prefixes for troubleshooting or modding
- Offer both CLI and GUI workflows in a single binary
- Remain lightweight with minimal dependencies
- Serve as a foundation for more advanced prefix management features in the future

This project is released under the MIT license. Contributions are welcome!

