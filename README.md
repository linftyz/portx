# portx

Modern port and process management for macOS and Linux.

`portx` is a terminal-first tool for inspecting listening ports, mapping them back to processes, reviewing exposure scope, and taking action quickly. It is intended as a friendlier alternative to bouncing between `netstat`, `lsof`, and `ps`.

[简体中文说明](./README.zh-CN.md)

## Status

`portx` is currently focused on macOS and Linux.

Implemented today:

- `list` to show listening ports
- `info` to inspect a specific port
- `find` to search by process name
- `kill` to terminate the process behind a port
- `watch` to refresh port details every second
- `tui` for an interactive terminal view
- `--json` output for `list`, `info`, and `find`

Planned later:

- Windows support
- More TUI polish and navigation
- Better automation integrations on top of the JSON contract

## Why portx

- Clear exposure scope detection: `PUBLIC`, `LAN`, `LOCAL`
- Port-oriented workflows instead of raw socket dumps
- Process-aware output with PID, name, command, user, and resource usage
- Friendly text output for humans and stable JSON output for tools
- Interactive TUI for live inspection

## Supported Scope Detection

`portx` classifies listeners by where they are reachable from:

- `PUBLIC`: `0.0.0.0`, `::`, and other globally reachable addresses
- `LAN`: `10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16`, `fc00::/7`, `fe80::/10`
- `LOCAL`: `127.0.0.0/8`, `::1`

Warnings are emitted for public wildcard or public global binds so risky listeners stand out quickly.

## Installation

### Download a prebuilt binary

When GitHub Releases are available, download the archive for your platform, extract it, and move `portx` into a directory on your `PATH`.

Planned release artifact names:

- `portx-vX.Y.Z-aarch64-apple-darwin.tar.gz`
- `portx-vX.Y.Z-x86_64-apple-darwin.tar.gz`
- `portx-vX.Y.Z-x86_64-unknown-linux-gnu.tar.gz`

### Build from source

```bash
git clone <your-repo-url>
cd portx
cargo build
```

### Run directly

```bash
cargo run -- list
```

### Install locally

```bash
cargo install --path .
```

### Install from a release archive

Example for macOS or Linux after downloading an archive:

```bash
tar -xzf portx-vX.Y.Z-<target>.tar.gz
cd portx-vX.Y.Z-<target>
chmod +x portx
mv portx /usr/local/bin/portx
```

You may choose a different destination as long as it is on your `PATH`.

## Usage

```bash
portx
portx list [--scope public|lan|local] [--json]
portx <port>
portx info <port> [--pid <pid>] [--json]
portx find <process_name> [--scope public|lan|local] [--json]
portx kill <port> [--pid <pid>] [--force] [--yes]
portx watch <port> [--pid <pid>]
portx tui
```

### Notes

- `portx` without a subcommand behaves like `portx list`.
- `portx 3000` is normalized to `portx info 3000`.
- `kill` is conservative by default:
  - it sends `SIGTERM` unless `--force` is used
  - it asks for confirmation in interactive terminals
  - in non-interactive contexts you must pass `--yes`
  - if a port has multiple owning PIDs, you must pass `--pid`
- Some process metadata may be unavailable depending on OS permissions. In that case, text output shows `N/A` and JSON uses `null`.

## Examples

### List all listeners

```bash
portx
```

### Show only local listeners

```bash
portx list --scope local
```

### Inspect port 5432

```bash
portx info 5432
```

### Inspect a specific PID on a shared port

```bash
portx info 3000 --pid 4242
```

### Search for all Node.js listeners

```bash
portx find node
```

### Use JSON in scripts

```bash
portx list --json
portx info 5432 --json
portx find postgres --json
```

### Gracefully stop the process behind a port

```bash
portx kill 3000
```

### Force kill without a prompt

```bash
portx kill 3000 --pid 4242 --force --yes
```

### Watch a port live

```bash
portx watch 5432
```

### Open the interactive TUI

```bash
portx tui
```

## TUI Shortcuts

- `Up` / `Down`: move selection
- `Enter`: toggle detail focus
- `k`: open kill confirmation
- `y` / `n`: confirm or cancel kill
- `?` / `h`: open or close help
- `Esc`: leave focused detail mode or close overlays
- `q`: quit

The TUI requires an interactive terminal.

## JSON Output

`list`, `info`, and `find` support `--json`.

Design goals:

- stable field names for automation
- shared service-layer data for text and JSON output
- partial data is preserved instead of failing the whole command

When a value cannot be collected:

- text output shows `N/A`
- JSON output uses `null`

This makes the JSON contract a good base for future wrappers such as editor integrations, shell workflows, or small automation services.

## Architecture

The project is organized into three broad layers:

- `src/platform`: platform-specific socket and process collection
- `src/core`: shared models, scope detection, warnings, and service logic
- `src/output` and `src/tui`: CLI presentation, JSON output, text formatting, and terminal UI

This keeps collection concerns separate from user-facing presentation and makes later platform expansion easier.

## Development

Useful commands while working on the project:

```bash
cargo fmt --check
cargo clippy
cargo test
```

See also:

- [CHANGELOG.md](./CHANGELOG.md)
- [LICENSE](./LICENSE)

## Distribution

For other users, the most practical distribution options are:

- source install with `cargo install --path .`
- prebuilt archives from GitHub Releases

This repository includes a GitHub Actions workflow that builds release archives for:

- Apple Silicon macOS
- Intel macOS
- x86_64 Linux

Pushing a tag like `v0.1.0` will build and upload release assets automatically.

## Current Limitations

- macOS and Linux only
- Windows is not part of the current implementation
- Some process and user fields depend on system permissions
- The TUI is intentionally lightweight in this phase and will continue to evolve

## Roadmap

- Windows support
- More TUI navigation and detail ergonomics
- Improved filtering and sorting in the interactive view
- Better packaging and release workflow
- Optional higher-level integrations built on top of JSON output
