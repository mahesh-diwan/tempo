# Tempo

Tempo is a privacy-respecting, anonymous YouTube music player for the desktop,
built in Rust with an mpv playback backend.

## Purpose

Play and organize YouTube music locally without accounts, ads, or tracking.

## Build prerequisites

- Rust (stable toolchain; see `rust-toolchain.toml`)
- `mpv`
- `libmpv` (development headers)

## Anonymous v1 scope

The first release supports anonymous use only: no login, no account linking,
no cloud sync. Playlists and library data are stored locally.
