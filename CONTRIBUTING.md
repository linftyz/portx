# Contributing to portx

Thanks for your interest in contributing to `portx`.

This project is being built incrementally. The goal is not just to add features quickly, but to keep the codebase easy to understand, easy to extend, and safe to evolve across milestones.

## Project Priorities

When contributing, optimize for these goals:

- Keep the CLI stable and predictable
- Prefer small, reviewable changes over large rewrites
- Preserve a clean separation between collection, service logic, and presentation
- Make the tool pleasant to use, not just technically correct
- Keep macOS and Linux behavior aligned where practical

## Current Scope

`portx` currently targets:

- macOS
- Linux

Windows support is planned later and should not block current progress unless a change makes future support harder.

## Development Setup

### Requirements

- Rust toolchain
- Cargo
- A macOS or Linux environment for runtime testing

### Getting started

```bash
git clone <your-repo-url>
cd portx
cargo build
```

## Recommended Workflow

The project works best when changes are made in small milestones.

1. Pick one focused change.
2. Keep the repository compiling throughout the work.
3. Run formatting, linting, and tests before finishing.
4. Stop at a natural checkpoint and commit.

Good examples:

- add one command behavior
- improve one output mode
- polish one TUI interaction
- add one group of tests

Less ideal examples:

- rewriting multiple layers at once
- mixing platform work, output changes, and refactors in one commit
- changing JSON contracts casually

## Architecture Guidelines

The codebase is intentionally split by responsibility:

- `src/platform`
  - OS-specific socket and process collection
- `src/core`
  - shared models, scope detection, warnings, service logic
- `src/output`
  - text and JSON presentation for CLI commands
- `src/tui`
  - interactive terminal UI
- `src/cli` and `src/commands`
  - argument parsing and command dispatch

Please try to keep changes in the right layer.

Examples:

- If you are changing how listeners are collected, prefer `src/platform`.
- If you are changing sort/filter/warning behavior, prefer `src/core`.
- If you are changing how information is rendered, prefer `src/output` or `src/tui`.

## Command and Output Design

When adding or changing behavior:

- prefer explicit, readable CLI output
- avoid breaking existing JSON field names unless absolutely necessary
- use `N/A` in text output and `null` in JSON when data is unavailable
- treat multi-PID operations conservatively
- preserve the principle that risky actions should require clear intent

## Testing Expectations

Before opening a change, run:

```bash
cargo fmt --check
cargo clippy
cargo test
```

Tests are especially valuable for:

- scope classification
- sort and filter behavior
- warning generation
- JSON output shape
- command normalization
- kill safety rules

If a change is hard to test automatically, explain the manual verification you performed.

## Commit Style

Small, focused commits are strongly preferred.

Examples from the existing history:

- `feat: list listening ports`
- `feat: add json output`
- `feat: kill process by port`
- `refactor: polish tui layout`
- `docs: add project readmes`

Recommended prefixes:

- `feat:`
- `fix:`
- `refactor:`
- `test:`
- `docs:`
- `chore:`

## Pull Request Notes

If you open a PR, it helps to include:

- what changed
- why the change is needed
- any command or output contract affected
- how you verified it
- screenshots or terminal output snippets for TUI / CLI presentation changes when useful

## Areas That Need Help

Useful contribution areas include:

- Windows support
- better TUI navigation and scrolling
- packaging and installation improvements
- stronger cross-platform verification
- output polish and ergonomics
- documentation and examples

## Questions and Proposals

If you want to propose a larger change, prefer discussing the shape first instead of landing a broad rewrite immediately. The project values forward-looking structure, but it also values keeping momentum and preserving working checkpoints.

If your change affects versioning or release planning, also check [RELEASING.md](./RELEASING.md).
