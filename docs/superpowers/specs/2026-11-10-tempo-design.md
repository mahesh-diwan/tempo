# Tempo — native YouTube Music streaming client

Date: 2026-11-10
Status: draft for approval
Project: `tempo`, new repository at `github.com/mahesh-diwan/tempo`
Supersedes the port design in
`2026-11-10-native-rust-youtube-streaming-design.md`.

## Intent

Build a new, independent desktop music player called **Tempo**. Its first and
only v1 job is to **search YouTube Music and stream audio**, natively, without a
browser engine or a JavaScript toolchain.

Tempo is greenfield. It starts from an empty workspace. The existing Ryotunes
repository is a read-only reference: proven modules may be copied, but no code
is shared, no build is shared, and no identity is inherited.

## What "greenfield" means here

- New repository, new workspace, new version line at `0.1.0`.
- New app identity, new data directory, new database.
- No import of the Ryotunes database, settings, cookies, or account data.
- No Ryoku naming, theme files, or integration.
- Copy allowed: the libmpv wrapper, the direct-URL InnerTube client code, queue
  transition rules, and the SQLite access patterns. Copied files get rewritten
  to fit the new crate boundaries.
- Rewritten from zero: application wiring, state machine boundary, UI, theme,
  packaging, CI.

## Goals

1. Launch a native window and search YouTube Music.
2. Play audio through libmpv with pause, seek, volume, next, previous.
3. Keep a queue: play now, play next, add, remove, reorder.
4. Show now-playing metadata and artwork.
5. Standalone dark and light theme that follows the desktop preference.
6. Keep playing with the window closed, via MPRIS and a tray icon.
7. Package for Arch with no Node, WebKit, GTK, or Ryoku runtime dependency.
8. Be measurably lighter than the existing Ryotunes build.

## Non-goals (v1)

- Account sign-in, cookies, account switching.
- Liked songs, library, account playlists, history, stats, uploads, ratings.
- Internet radio, Discord Rich Presence, Listen Together, Last.fm.
- Local music library, lyrics, mini player.
- Music-video playback. Audio only.
- Windows and macOS packaging.
- Ryoku theme integration.
- Pixel parity with the Ryotunes Svelte UI.

## Anti-goals

- Keeping a WebKit window for login or token generation.
- Rewriting the audio engine before the first stream plays.
- Importing or reading the Ryotunes database.
- Depending on the Ryotunes repository at build or run time.
- Requiring users to install Node, pnpm, or a browser engine.

## Constraints

- Linux x86_64 first; Wayland primary, X11 acceptable.
- libmpv for v1 audio. It is C, but it is the proven path.
- GPL-3.0-or-later, required because proven GPL code is copied.
- YouTube extraction is unofficial and can change without notice. Failure
  degrades to a clear error, never a crash.
- Anonymous mode only. No secrets or cookies are stored.
- Target: usable, not feature-complete.

## Approaches considered

### A. iced + libmpv + anonymous search/play (chosen)

- Wins: no web stack, proven audio, shortest path to a first stream.
- Costs: iced UI written from scratch; extraction remains the schedule risk.

### B. iced + Symphonia + cpal

- Wins: no C audio dependency.
- Costs: rebuild buffering, gapless, seek, headers, speed/pitch. Several extra
  weeks and higher quality risk. Deferred behind the `player` API.

### C. GTK4 / Relm4 + libmpv

- Wins: faster widget parity.
- Costs: keeps a large C UI stack and much of the memory footprint.

### D. Web UI with a smaller shell

- Rejected: still ships a browser engine and the JavaScript toolchain.

## Architecture

```text
apps/tempo        thin main: runtime, wiring, shutdown
   │
   ├── crates/ui        iced window, widgets, input, theme
   │        ▲ CoreEvent
   │        ▼ CoreCommand
   ├── crates/core      state machine, queue, search, settings, DB
   │        ├── crates/youtube   InnerTube search + resolve, rustypipe fallback
   │        ├── crates/player    libmpv wrapper
   │        └── crates/platform  MPRIS, tray, single instance
   └── crates/model     shared plain data types, no I/O
```

`crates/core` owns playback and queue state. It has no iced, GTK, WebKit, or
window dependency.

### State and event boundary

```rust
pub enum CoreEvent {
    SearchStarted { query: String },
    SearchResults { query: String, items: Vec<SongItem> },
    SearchFailed { query: String, message: String },
    QueueChanged(QueueSnapshot),
    NowPlaying(NowPlaying),
    PlaybackState { playing: bool },
    Position(f64),
    Duration(f64),
    Volume(i64),
    PlaybackError { message: String },
}

pub trait EventSink: Send + Sync {
    fn emit(&self, event: CoreEvent);
}
```

The UI sends commands (`Search`, `Play`, `PlayIndex`, `TogglePause`, `Seek`,
`SetVolume`, `Next`, `Previous`, `AddToQueue`, `PlayNext`, `RemoveFromQueue`,
`MoveInQueue`, `Shutdown`). Core emits events back. No JSON bridge.

### Crates

- `crates/model`: `SongItem`, `QueueSnapshot`, `NowPlaying`, IDs. No I/O.
- `crates/youtube`: search, resolve, direct-client fallback. Copied and
  restructured from the Ryotunes `innertube` crate.
- `crates/player`: libmpv wrapper. Copied from the Ryotunes `player` crate.
- `crates/core`: queue, playback orchestration, settings, SQLite, stream cache.
- `crates/platform`: MPRIS, tray, single instance, desktop integration.
- `crates/ui`: iced views and theme. No business logic.
- `apps/tempo`: process entry, directories, logging, signals.

## Extraction strategy

Search uses the unauthenticated InnerTube search path.

Playback resolution order:

1. Direct-URL clients: `VISIONOS` → `ANDROID_VR_1_65_10` →
   `ANDROID_VR_1_43_32`.
2. `rustypipe` fallback.

`IOS` is excluded: YouTube serves only bounded-Range URLs to it and mpv never
makes Range requests. No `WEB_REMIX` cipher or PoToken path. No embedded
JavaScript engine. No cookie import.

The resolver returns stream URL, required headers, itag, expiry, loudness,
title, artist, duration, thumbnail, and the client that produced it.

### Milestone 2 gate

- Sample: 50 tracks from 10 search queries, popular and niche artists.
- Pass: results for all 10 queries, and at least 90 % of the 50 tracks resolve
  and play for 10 seconds or more.
- Fail: stop and report per-track causes.
- Below the gate: ship limited coverage. Do not start token work, a JS engine,
  or cookie import in v1. Expanding scope is a new decision made with evidence.

## Playback behavior

- Audio only: mpv `vid=no`, gapless on.
- Loudness normalization applied by default when the resolver supplies a gain.
  No user-facing setting in v1.
- The cache directory is mpv's demuxer cache only. No offline downloads.

## Queue

- Clicking a search result replaces the queue with the current result list and
  plays the clicked row.
- Shuffle and repeat off/all/one are in v1.
- At queue end, playback stops. No radio or autoplay continuation.
- MPRIS exposes shuffle and repeat where the platform supports them.

## UI (iced)

One window:

- Custom titlebar: drag region, `Tempo`, window controls.
- Search field, Enter to search, result count.
- Search results: virtualised rows with artwork, title, artist, duration.
- Player bar: artwork, title/artist, play/pause, previous/next, seek, volume.
- Queue panel: current track, upcoming tracks, remove, reorder.
- Empty, loading, and error states.
- Keyboard: space, arrows, enter, escape.

No mini player, lyrics panel, or account menu in v1.

## Theme

Matugen-driven, Material 3. No Ryoku files and no Ryoku naming.

Tempo reads `~/.config/matugen/generated/tempo.json`. A Tempo template in the
user's matugen config generates it on every wallpaper or palette change:

```toml
[templates.tempo]
input_path  = '~/.config/matugen/templates/tempo.json'
output_path = '~/.config/matugen/generated/tempo.json'
```

The template exports both modes (`dark`, `light`) with the Material 3 roles
Tempo uses: `primary`, `on_primary`, `primary_container`,
`on_primary_container`, `secondary`, `tertiary`, `surface`,
`surface_container_low`, `surface_container`, `on_surface`,
`on_surface_variant`, `outline`, `error`, `on_error`.

- Tempo watches the generated file with inotify and reloads live. No polling,
  no restart.
- The active mode follows the desktop color-scheme preference. A settings
  override selects dark or light explicitly.
- Missing or invalid file: fall back to built-in dark and light palettes. Never
  fail to start because of a theme file.
- The file is read-only input. Tempo never writes into `~/.config/matugen/`.
- The theme adapter sits behind one `ThemeSource` trait, so a different palette
  provider can replace matugen later.

## Lifecycle

- Closing the window hides the UI and keeps core, libmpv, MPRIS, and the tray
  alive while playing.
- Tray click or a second launch reopens the window from core state.
- Tray-only with nothing playing exits after a bounded idle period.
- Quit stops playback, unregisters MPRIS, and flushes state.
- Queue and settings persist in SQLite and restore on launch.

## Integrations

- MPRIS: metadata, play/pause, next/previous, seek, loop, shuffle.
- Tray: show, play/pause, next, previous, quit.
- Single instance: D-Bus name or lock file.
- No Discord, Last.fm, Listen Together, radio, or autostart in v1.
- No Ryoku integration.

## Persistence

New schema, new database. The Ryotunes database is never opened.

- Data: `$XDG_DATA_HOME/io.github.mahesh-diwan.tempo/tempo.sqlite`
- Cache: `$XDG_CACHE_HOME/io.github.mahesh-diwan.tempo/audio/`
- Permissions: 0700 directories, 0600 database.
- Tables: `settings`, `queue`, `stream_cache`. Migrations are versioned from
  the first release.

## Identity

| Field | Value |
|---|---|
| Repo | `github.com/mahesh-diwan/tempo` |
| Binary | `tempo` |
| Package | `tempo` |
| App ID | `io.github.mahesh-diwan.tempo` |
| MPRIS name | `tempo` |
| Window title | `Tempo` |
| Desktop file | `tempo.desktop` |
| License | `GPL-3.0-or-later` |
| First version | `0.1.0` |

## Error handling

- Search failure: visible message with retry.
- Resolution failure: skip or stop with a message naming the track.
- Playback failure: mpv error translated to user language; next track offered.
- Network loss: current track continues from buffer; retry on the next action.
- No panics on the UI thread from extraction or playback errors.

## Performance targets

| Metric | Ryotunes baseline | Tempo target |
|---|---|---|
| PSS, tray + playing | measure in Milestone 1 | ≤ 100 MiB |
| PSS, window open + playing | measure in Milestone 1 | ≤ 150 MiB |
| Browser engine processes | several | zero |
| Permanent timers | WebKit/JS + mpv | mpv events only |
| Cold start to interactive | measure | ≤ 1 s |
| Threads | measure | ≤ 20 |
| Node/WebKit/GTK runtime deps | required | none |

The baseline is recorded for comparison. Tempo targets stand on their own; a
missed target ships with the measurement and an explanation.

## Packaging

- PKGBUILD `tempo`, built from the new repository.
- Dependencies: `mpv`, Wayland/X11 and font libraries required by iced/wgpu.
- No `webkit2gtk`, `libappindicator`, `nodejs`, or `pnpm`.
- Desktop file, icon, and install paths follow the identity table.
- CI: Rust only; install libmpv; `cargo test --workspace`; release build.
- AUR publishing is out of scope; local build and install verification only.

## Milestones

1. **Scaffold and baseline.** New repo, workspace, crates, CI, old-app baseline
   numbers.
2. **Extraction spike.** Anonymous search plus play, no web stack. Gate above.
3. **Core.** Queue, playback orchestration, settings, persistence.
4. **UI shell.** Window, standalone theme, search and results.
5. **Playback UI.** Player bar, queue panel, now playing, keyboard.
6. **Integrations.** MPRIS, tray, lifecycle, single instance.
7. **Packaging.** PKGBUILD, desktop file, icon, install test, CI release.
8. **Measure and release.** Compare against baseline, fix, tag `0.1.0`.

Estimated size: **medium project, 4–6 weeks** for one experienced developer.
Milestone 2 is the schedule gate.

## Risks and mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Anonymous coverage is low | fewer playable tracks | Spike first; measure 50 tracks; ship limited coverage with per-track causes; no token work in v1 |
| YouTube changes extraction | playback breaks | one resolver API; direct-client fallbacks; feature-gated live tests |
| iced text/IME/accessibility gaps | weak search UX | test early; keep text inputs simple; revisit only with evidence |
| wgpu on NVIDIA/Wayland | blank window or crash | test early; select GL or Vulkan backend; document fallback |
| Copied code drags old assumptions | hidden coupling | copy module by module, rewrite boundaries, delete unused paths |
| Scope creep toward parity | delay | v1 checklist is fixed by this document |

## What is left to the builder

- Exact iced widget tree, spacing, and visual composition.
- Animation set that fits native rendering.
- Search result ranking and merging.
- Queue persistence granularity.
- Test fixtures and the live-smoke sample.
- Concrete crate dependency versions.

## Resolved decisions

1. Greenfield repository, start at `0.1.0`. Copy proven modules only.
2. Name `Tempo`; identity `io.github.mahesh-diwan.tempo`; no Ryoku strings.
3. Matugen theme; full Material 3 JSON at
   `~/.config/matugen/generated/tempo.json`, watched live. No Ryoku files.
4. No migration. The Ryotunes database is never opened or read.
5. Direct-URL clients only: `VISIONOS`, `ANDROID_VR_1_65_10`,
   `ANDROID_VR_1_43_32`, then `rustypipe`. No IOS, cipher, PoToken, JS engine,
   or cookies.
6. Below the gate: limited coverage ships; expansion is a new decision.
7. Search runs on Enter only.
8. Clicking a result replaces the queue with the current results.
9. Shuffle and repeat are in v1; playback stops at queue end.
10. Packaging is local verification only; no AUR publish.
11. First subagent run covers the plan, Milestone 1, and Milestone 2, then
    stops for the gate report.

## Approval

Approval approves the greenfield approach, iced plus libmpv, anonymous scope,
the identity table, the standalone theme, and the milestone order.
Implementation planning follows in `docs/superpowers/plans/` in the new
repository.
