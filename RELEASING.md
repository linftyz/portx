# Releasing portx

This document explains how `portx` thinks about versions and how to cut a formal release when the project is ready.

## Versioning Model

`portx` follows semantic versioning in spirit, with one important practical note:

- a version number in `Cargo.toml` is the current project version
- a version entry in `CHANGELOG.md` is a documented milestone
- neither one automatically means a release has been published

For example, `0.1.0` currently describes the first usable project baseline. It does **not** by itself imply that:

- a Git tag has been created
- a GitHub release exists
- the crate has been published anywhere

## Milestone vs Formal Release

It helps to separate these two ideas:

### Milestone version

Used when you want to:

- mark a coherent development checkpoint
- summarize what is implemented so far
- keep the changelog readable while the project is still evolving quickly

### Formal release

Used when you want to:

- publish a tagged version for others to install or reference
- announce a stable checkpoint externally
- create a reproducible release artifact

You can have milestone versions before you start doing formal releases.

## Current State

At the moment:

- `Cargo.toml` is set to `0.1.0`
- `CHANGELOG.md` contains a `0.1.0` baseline entry
- this should be read as a project baseline, not as proof of publication

## Recommended Release Checklist

When you decide to do a formal release, this is a good lightweight workflow:

1. Confirm the working tree is clean.
2. Run:

```bash
cargo fmt --check
cargo clippy
cargo test
```

3. Review `CHANGELOG.md` and move any completed items out of `Unreleased`.
4. Decide the release version.
5. Update version references if needed:
   - `Cargo.toml`
   - `CHANGELOG.md`
   - README or docs if they mention the version explicitly
6. Commit the release preparation.
7. Create a Git tag such as `v0.1.0`.
8. Push the commit and tag.
9. Optionally create a GitHub release and attach release notes.
10. Optionally publish binaries or the crate if you choose to distribute it that way.

## Choosing the Next Version

Some rough guidelines:

- `0.1.x`
  - small fixes, output polish, docs, minor compatibility improvements
- `0.2.0`
  - meaningful feature additions or command/output changes
- `0.x.0`
  - still pre-1.0, but broader changes are expected
- `1.0.0`
  - when the core command behavior and output contracts feel intentionally stable

## Practical Note

While the project is still moving quickly, it is completely reasonable to:

- keep using milestone versions in the changelog
- avoid cutting external releases until the CLI and JSON contract feel settled

That is the current posture of `portx`.
