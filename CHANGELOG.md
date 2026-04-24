# Changelog

All notable changes to this project will be documented in this file.

The format is inspired by [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), adapted for the current stage of the project.

## [Unreleased]

### Added

- Placeholder for upcoming changes.

## [0.1.0] - 2026-04-24

Initial public project baseline for `portx`.

### Added

- Core CLI commands: `list`, `info`, `find`, `kill`, `watch`, and `tui`
- macOS and Linux listener collection
- Process association with PID, command, cwd, user, CPU, memory, thread count, uptime, and connection count where available
- Exposure scope detection for `PUBLIC`, `LAN`, and `LOCAL`
- Security warnings for public wildcard and public global binds
- JSON output support for `list`, `info`, and `find`
- Interactive TUI with live refresh, detail view, kill confirmation, and help overlay
- Project README in English and Simplified Chinese

### Changed

- Improved CLI text presentation and column alignment
- Clarified JSON output semantics for partial and unavailable fields
- Polished TUI layout with better hierarchy, scope counts, and grouped detail sections

### Testing

- Added core unit and integration-style coverage for scope detection, filtering, sorting, kill safety checks, JSON output, and command normalization

## Release Notes

At the current stage, `portx` is intentionally focused on:

- macOS and Linux first
- stable command behavior before feature breadth
- readable terminal output for humans
- structured JSON output for future automation

Windows support is planned for a later phase.
