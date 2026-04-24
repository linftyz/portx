# Changelog

All notable changes to this project will be documented in this file.

The format is inspired by [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), adapted for the current stage of the project.

## [Unreleased]

- No unreleased changes yet.

## [0.1.0] - 2026-04-25

- Core CLI commands: `list`, `info`, `find`, `kill`, `watch`, and `tui`
- macOS and Linux listener collection
- Process association with PID, command, cwd, user, CPU, memory, thread count, uptime, and connection count where available
- Exposure scope detection for `PUBLIC`, `LAN`, and `LOCAL`
- Security warnings for public wildcard and public global binds
- JSON output support for `list`, `info`, and `find`
- Interactive TUI with live refresh, detail view, kill confirmation, and help overlay
- Project README in English and Simplified Chinese
- Simplified project docs for a solo-maintained workflow
- Added release workflow for prebuilt macOS and Linux archives
- Improved CLI text presentation and column alignment
- Added grouped listener output for paired IPv4 and IPv6 binds in `list` and `find`
- Added color hierarchy and shared output styling helpers for clearer CLI hierarchy
- Clarified JSON output semantics for partial and unavailable fields
- Improved human-friendly error output with follow-up guidance for common failures
- Polished `kill` confirmation and result output for safer process termination flows
- Polished TUI layout with better hierarchy, scope counts, and grouped detail sections
- Improved TUI detail scrolling, focus behavior, and help visibility
- Added core unit and integration-style coverage for scope detection, filtering, sorting, kill safety checks, JSON output, and command normalization
