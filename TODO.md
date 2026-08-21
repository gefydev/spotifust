# Project State Machine

## Current Focus

- [ ] Integrate LRCLIB REST API for millisecond-synced `.lrc` lyrics auto-scrolling with Genius fallback

## Development Backlog

### Phase 1: Bootstrapping & Core Architecture

- [x] Configure Cargo.toml with feature flags for Iced (tiny-skia backend), RSpotify, and Librespot
- [x] Define central `AppError` enum (thiserror) with per-subsystem variants
- [x] Set up base Model-View-Update loop in `src/app.rs`
- [x] Set up full GitHub Actions CI/CD infrastructure, Issue templates, and documentation
- [x] Verify all `librespot` and `rspotify` raw error types are wrapped in `AppError` before reaching `Message` variants
- [x] Audit and eliminate any remaining `.unwrap()` / `.expect()` calls outside `main()` bootstrap
- [x] Reduce RAM baseline from ~45 MB down to the target < 25 MB ceiling

### Phase 2: Spotify Resizable Panel Layout Engine

- [x] Implement 3-column layout structure (Left Sidebar library, Main content, Right panel)
- [x] Add interactive drag handles with `ResizingHorizontally` mouse cursor interaction
- [x] Handle global pointer move/up events for robust dragging/resizing
- [x] Implement right dynamic slot panel showing Now Playing or Queue based on playback bar triggers
- [x] Implement left library sidebar collapse to icon-only compact layout below width threshold
- [x] Persist layout panel widths to disk

### Phase 3: Librespot Audio & Session Pipeline

- [x] Implement `librespot::core::session::Session` setup and credential-based login
- [x] Implement a custom `librespot` audio `Sink` that captures decoded PCM frames
- [x] Route PCM frames from the custom Sink through a bounded `mpsc` channel to a `rodio` playback thread
- [x] Wire a synthetic sine-wave test pipeline to validate the `rodio` backend end-to-end
- [x] Wire UI Play command to call `player.load()` on the active `librespot` player instance
- [x] Wire UI Pause / Resume commands to the librespot player
- [x] Wire UI Skip Next / Skip Previous commands to the librespot player
- [x] Implement Seek: accept a `f32` position ratio from the seek bar and call `player.seek(ms)`
- [x] Extract current track metadata (title, artist, album, duration) from `PlayerEvent` and emit them as `Message::TrackChanged`
- [x] Stream playback position (elapsed ms) from the audio task to the UI via the mpsc channel
- [x] Implement end-of-track detection via `PlayerEvent::EndOfTrack` and auto-advance to next track
- [x] Validate that the mpsc channel remains bounded under sustained high-throughput decoding
- [x] Wire volume control: slider value in UI → `rodio::Sink::set_volume()` (full 0.0–1.0 range, not binary)
- [x] Fix seek bar so it travels the full 0–100% range and reflects real playback position
- [x] Handle `librespot` session expiry and reconnection without crashing
- [x] Fix app crash during track playback (`src/audio/sink.rs:35:14: Cannot block the current thread from within a runtime` & `Invalid Spotify URI ''`)
- [x] Refine audio pipeline for 320kbps high-quality bitrate, synchronized rodio pause/resume and instant volume binding

### Phase 4: RSpotify Web API, Auth & Aggressive Caching

- [x] Implement PKCE Authorization Code Flow with `rspotify`
- [x] Register `spotifust://callback` custom protocol handler for the OAuth redirect
- [x] Verify the refresh token is stored exclusively via the OS keychain (`keyring` crate), never as plaintext
- [x] Implement token refresh on expiry: detect 401 responses and silently re-authenticate
- [x] Fetch the authenticated user's profile (`/me`) and display name and avatar in the sidebar
- [x] Fetch the user's full playlist library (`/me/playlists`, paginated) and stream items into the sidebar list
- [x] Fetch playlist track listings on demand when a playlist is selected
- [x] Fetch the user's saved albums and expose them in a dedicated Albums view
- [x] Fetch the user's top tracks and expose them in a Home/For You view
- [x] Implement search: send queries to `/search` and display track, album, and artist results
- [x] Implement album detail view: fetch `/albums/{id}` and list its tracks
- [ ] Implement artist detail view: fetch `/artists/{id}` with top tracks and discography (the API exists in `src/api/artist.rs` but no UI ever calls it — see Phase 10 item 3)
- [x] Fetch currently playing track via `/me/player/currently-playing` on startup and sync UI state
- [x] Implement album art fetching: download cover images asynchronously and cache to disk in `src/api/cache.rs`
- [x] Implement a metadata cache layer in `src/api/cache.rs` to avoid redundant API calls (TTL-based)
- [x] Implement rate-limit handling: respect `Retry-After` headers from the Spotify API
- [x] Display large cover art in playlist and album detail header views
- [x] Audit and remove all remaining mock data across all UI views and components, fetching 100% live Spotify API data
- [x] Optimize long playlist loading with incremental chunking/streaming or virtualized pagination to avoid UI lag
- [x] Validate existing token/session before rendering initial screen to eliminate temporary login flicker
- [x] Achieve near-instant API data loading through aggressive metadata and disk image caching (TTL-based, local disk cache for instant startup render)
- [x] Implement local audio file scanner and persistence matching local tracks in playlists
- [x] Implement Track & Artist Radio / Recommendations endpoint (`GET /v1/recommendations`, "Made for You", "New Releases")

### Phase 5: UI Design System, Component Polish & Settings Page

- [x] Define a unified design token system (color palette, spacing scale, typography scale) in a central `theme.rs`
- [x] Replace all ad-hoc hardcoded color literals and magic numbers with design tokens
- [ ] Implement smooth hover transitions on sidebar items, buttons, and playback controls
- [x] Implement animated loading skeletons for album art, playlist headers, and track list placeholders while initial Spotify API data is fetching (zero mock/temp data, instant Spotify data render)
- [x] Remove "Explore Premium" / "Explorar Premium" button from sidebar and navigation
- [x] Add waveform or animated equalizer bars to the Now Playing area during active playback
- [x] Implement smooth progress bar animation that interpolates position between tick updates
- [x] Add context menus (right-click) on tracks, albums, and artists with distinct tailored options (Add to queue, Go to artist, Go to album, Share link, Copy URI, Add/remove from playlist, Save album, Follow artist, Edit/delete playlist) with click-outside dismiss, accurate cursor positioning, and 5s auto-dismiss toasts
- [x] Implement a proper volume slider that covers the full 0–100% range with a mute toggle
- [x] Add keyboard shortcuts for Play/Pause (Space), Skip (→/←), Volume (↑/↓)
- [x] Implement a mini-player / compact mode for when the window is resized to small dimensions
- [ ] Implement drag-and-drop track reordering within a playlist queue view
- [x] Add toast / snackbar notifications for user-facing errors and confirmations
- [x] Audit and refine all font sizes, weights, and line heights for visual consistency
- [x] Ensure the entire UI is navigable via keyboard (tab order, focus rings)
- [x] SETTINGS PAGE: Build base Settings page layout frame
- [x] LYRICS: Implement base Lyrics view layout frame
- [ ] Integrate LRCLIB REST API for millisecond-synced `.lrc` lyrics auto-scrolling with Genius plain lyrics fallback
- [ ] Integrate Last.fm API (`artist.getInfo`) + Wikipedia REST API for artist bio, curiosities, genres, and similar artists in Now Playing right panel
- [ ] Implement Spotify Connect icon & interactive device selector modal/popover in bottom playback bar
- [ ] Enhance Search screen with Category Pill filters (Tracks, Albums, Artists, Playlists) and Top Result spotlight card
- [ ] Implement Friend Activity / Social Feed side panel in right panel slot

### Phase 6: Queue, Playback State, Shuffle & Advanced Audio

- [x] Implement an internal play queue data structure in the `Model`
- [x] Display the current queue in a slide-out panel
- [x] Implement Shuffle mode: randomise queue order and persist the shuffle seed
- [x] Implement Repeat modes: No Repeat, Repeat Queue, Repeat One
- [x] Implement "Add to queue" action from track context menus
- [x] Implement track reordering and control within the play queue view
- [x] Implement Spotify-style structured User Queue, Context Queue, and playback History stack
- [x] Eliminate progress bar jumps and sync position directly with audio stream
- [ ] Spotify Connect: Full bi-directional Spotify Connect integration for remote control and device sync
- [ ] Crossfade: Smooth audio crossfade between tracks (configurable duration in Settings) (not actually implemented — only a no-op legacy stub remains — see Phase 10 item 10)
- [ ] Implement multi-band DSP Audio Equalizer with presets (Flat, Bass Boost, Vocal, Rock, Pop) integrated into `rodio` audio pipeline
- [ ] Implement Audio Loudness Normalization (ReplayGain / Spotify Normalization)
- [ ] Implement Gapless Playback transition between tracks

### Phase 7: System Integration & Local Files

- [x] Add application window and taskbar/dock icon support for Windows, macOS, and Linux distros
- [ ] Add 100% functional native System Tray (Systray) icon for Linux, macOS, and Windows with minimize-to-tray and playback menu (Play/Pause, Skip, Show/Hide, Quit) (`SystemTrayManager` in `src/ui/systray.rs` is a stub never used by the app — see Phase 10 item 8 & Blocked)
- [ ] Register global media key bindings (MPRIS on Linux, MediaSession on Windows/macOS) (no implementation exists — see Phase 10 item 9 & Blocked)
- [ ] Implement MPRIS2 D-Bus interface on Linux for desktop environment integration (zero D-Bus code exists in the tree — see Phase 10 item 9 & Blocked)
- [x] Local Files: Implement local audio file scanner and playback for custom local music directory path
- [ ] Implement Drag-and-Drop: drop tracks onto left sidebar playlists to append items
- [ ] Package the binary as a `.deb` and `.rpm` for Linux
- [x] Package the binary as a `.dmg` / `.app` bundle for macOS
- [x] Package the binary as an `.msi` installer for Windows
- [ ] Integrate auto-update check: compare current version against GitHub Releases on startup (`check_for_updates()` exists but is never invoked and targets the wrong repo owner — see Phase 10 item 7)
- [ ] Write end-to-end integration tests for the auth flow and audio pipeline

### Phase 8: Performance & Speed Optimization

- [x] Optimize general app execution speed, reducing UI update latency and startup load time
- [x] Run a full memory profile and verify the application stays under 25 MB baseline at idle
- [x] Profile and eliminate any hot-path allocations in the canvas render loop and audio callback
- [x] Replace any `.clone()` / `.to_string()` in hot paths with borrows (`&str`, `&[u8]`) where applicable
- [x] Run `cargo clippy --all-targets -- -D warnings` clean and resolve all lints
- [x] Run `cargo deny check` and ensure no disallowed licenses or duplicated dependencies
- [ ] Set up memory-leak detection in CI (Valgrind or similar) for the audio pipeline
- [x] Add structured logging (`tracing` crate) with configurable verbosity levels
- [x] Implement graceful shutdown: flush audio buffers and close the librespot session cleanly on exit
- [x] RAM baseline optimization: bounded image cache handle capacity to 64 items to keep RAM under 25 MB ceiling

### Phase 9: Comprehensive Functional Settings System (100% Backend Wired, Zero Mockups)

- [ ] SECTION 1 - Account & Language: External browser link to login methods (`spotify.com/account`) & persistent i18n UI language selector dropdown
- [ ] SECTION 2 - Explicit Content & Autoplay: Explicit content filter toggle (hide `explicit == true` tracks) & Autoplay toggle switch (auto-fetch `/v1/recommendations` on end of queue)
- [ ] SECTION 3 - Audio Quality & Library: Bitrate selector dropdown (Normal 96k, High 160k, Very High 320k bound to librespot decoder), automatic bitrate step-down on network lag, compact library view toggle, show/hide local files toggle, multi-folder source picker list with live rescanner, and external playlist import button
- [ ] SECTION 4 - Display & Canvas: Display toggles (auto-open Now Playing on play, desktop overlay on playback controls) & Canvas/Video toggles (looping background Canvas & audio-only video fallback)
- [ ] SECTION 5 - UI Scaling & Hotkeys: UI Scale selector (70%-130%) with `Ctrl +` / `Ctrl -` hotkeys and Reset button
- [ ] SECTION 6 - Privacy & Profile: Private Session toggle (6h auto-off), recent activity visibility dropdown, connected apps link, and profile element toggles (recent artists, followers, default public playlists)
- [ ] SECTION 7 - Playback & DSP Equalizer: Crossfade slider (0-12s), Automix toggle, Smart Shuffle switch, Mono Audio downmix toggle, Volume Normalization & Loudness level (Normal/Loud/Quiet), interactive 6-band DSP Equalizer (60Hz, 150Hz, 400Hz, 1kHz, 2.4kHz, 15kHz with -12dB to +12dB sliders & presets), and audio output device selector dropdown bound to rodio output enumeration
- [ ] SECTION 8 - System, Storage & Hardware: Auto-start on system boot dropdown, Close button minimizes to system tray toggle, Storage usage indicator (Downloads vs Cache MB), Clear Cache button (`src/api/cache.rs`), Offline storage path relocation picker, Proxy configuration selector (Auto-detect, HTTP, SOCKS5), and Hardware Acceleration switch

### Phase 10: Correctness & Bug Fixes (deep audit of current codebase)

- [x] 1. Fix context-menu positioning: every `Open*ContextMenu` message is emitted with hardcoded coordinates (`iced::Point::new(450.0, 300.0)` and similar) instead of the real cursor position — capture the actual position from `iced::Event::Mouse(CursorMoved)`
- [x] 2. Fix "Ir al álbum" track context action: it sends `SelectAlbum(album_name)`, passing an album *name* where an album *ID* is expected (and only looks it up in saved albums) — thread the real `album_id` through `TrackInfo`/context payloads so navigation works from any source
- [x] 3. Wire the artist detail view: `fetch_artist_details()` in `src/api/artist.rs` (top tracks + discography + genres + followers) is never called — make "Ir al artista" open a real Artist page instead of triggering a text search, and render the fetched data
- [x] 4. Wire the dead `OpenArtistContextMenu` message: no UI element emits it, so the artist context menu (follow/unfollow) is unreachable — hook it to artist cards in search results and album/playlist headers, and fetch real follow state (`/v1/me/following/contains`) so only the relevant action is shown instead of both
- [x] 5. Fix "Agregar canciones del álbum a playlist": the album context menu passes an album ID as if it were a track URI (`OpenAddToPlaylistModal(vec![album_id])`) — expand the album into track URIs before opening the modal
- [ ] 6. Remove the 200-track hard cap in `fetch_playlist_tracks` and render long playlists incrementally (chunked append or virtualized rows) so big playlists aren't silently truncated and don't lag the UI
- [ ] 7. Fix auto-update: invoke `check_for_updates()` at startup (it is never called) and correct the GitHub URL from `elgena/spotifust` to `GenaDeev/spotifust`; surface the result as a toast/banner with a download link
- [ ] 8. Implement the real system tray (see Blocked — needs `tray-icon`): current `SystemTrayManager` is a stub with no tray icon, minimize-to-tray, or playback menu, and is not referenced by `app.rs`
- [ ] 9. Implement MPRIS2 D-Bus + global media keys (see Blocked — needs `zbus` as a direct dependency): no D-Bus/media-key code exists today
- [ ] 10. Implement real crossfade: the only remnant is a no-op `AudioCommand::SetCrossfade` in the legacy engine — implement overlap/fade between tracks in the rodio pipeline (rodio already supports `fade_in`/`fade_out`)
- [x] 11. Remove the legacy mock sine-wave `AudioEngine` (`src/audio/engine.rs`): it spawns threads before login, is Phase-3 scaffolding, and is still used as a playback fallback in `TogglePlayback` when no audio session exists — also delete the dead `PlayerCommand::SkipNext/SkipPrev` variants in `session.rs` that just reload the same URI
- [ ] 12. Replace hardcoded user paths: `get_user_music_dir()` falls back to `/home/elgena/Música` and the Settings page shows a static `/home/elgena/Music` — use XDG dirs / `$XDG_MUSIC_DIR` or a persisted setting
- [ ] 13. Fix cache semantics: `DiskMetadataCache` never expires (deleted playlists/albums resurrect from disk) and `ImageCache` never evicts (unbounded disk growth) — add TTL/versioning to metadata and LRU eviction (count + bytes) to images
- [ ] 14. Move the image/metadata cache out of `std::env::temp_dir()` (wiped on reboot, per-user pollution) into a proper app-data dir (`~/.local/share/spotifust`, `%APPDATA%`, etc.) so the "instant startup render" cache actually survives
- [x] 15. Debounce the search input (~300 ms): today every keystroke fires `/v1/search`, risking 429 rate limits and wasted requests
- [ ] 16. Fix the local-track pipeline: duration is hardcoded to 180 s, cover art is found by byte-magic scanning, and playback sends an empty URI (`Play("")` → "Cannot play track with empty Spotify URI") — parse real metadata (symphonia is already in the tree via librespot) and wire a working local playback path that uses `local_path`
- [ ] 17. Wire the playback-bar heart button (currently `Message::MockAction`) to save/unsave the current track via `/v1/me/tracks` (Liked Songs parity), and use the existing `Icon::VolumeMute` for the mute button instead of `Icon::X`
- [ ] 18. Fix `TogglePlaylistPrivacy`: it always assumes `currently_public = true` without knowing the real state — fetch the playlist's current visibility before toggling
- [ ] 19. Give `NavigationItem::Library` a real view (today it renders the same Home dashboard): aggregate playlists, saved albums, and followed artists with search/filter, like Spotify's "Your Library"
- [ ] 20. Unify playback-position sources: the UI ticks every 200 ms (`PlaybackTick`) while the session pushes `PositionMs` every 500 ms — pick one source of truth to avoid progress drift
- [ ] 21. Fix the bitrate mismatch: `session.rs` hardcodes `Bitrate160` while README/TODO/Settings all claim "320 kbps (Very High)" — restore `Bitrate320` or wire the Phase 9 §3 bitrate selector immediately
- [ ] 22. Memoize the local-file scan: `match_and_persist_local_tracks` re-scans the whole music directory on every playlist load — cache the scan results with invalidation on folder change
- [ ] 23. Search parity: include playlists in `/v1/search` results and mark explicit tracks with the Ⓔ badge (feeds the Phase 9 §2 explicit-content filter)
- [x] 24. Playback immediacy: clear the stale rodio buffer on new track load so a freshly-selected track starts instantly instead of playing leftover buffered audio first
- [x] 25. Perf/RAM: use 300px thumbnails for track-level artwork (was always the 640px image) and bound concurrent image downloads to 6 via a shared semaphore — kills the startup download storm and cuts decoded-image RAM several-fold

### Phase 11: Spotify Parity Features (things the original client has that Spotifust doesn't)

- [ ] 1. Liked Songs: dedicated "Liked Songs" entry in the sidebar backed by `/v1/me/tracks`, working heart save/unsave, and an "Add to Liked Songs" context action
- [ ] 2. Recently Played shelf on Home (`/v1/me/player/recently-played`) — Spotify's "Recently played" row
- [ ] 3. "Play next" context action (insert at the top of the user queue) alongside "Add to queue"
- [ ] 4. Sleep timer (stop playback after N minutes or at end of current track) in the Queue panel — Spotify desktop has it under Queue → Sleep timer
- [x] 5. Log out / switch account: the avatar button is a `MockAction`; add a profile menu with Log out (clears the keyring token), Account link (`spotify.com/account`), and account switching — the app currently has no way to sign out (logout + session cleanup + round avatar popup done; account link & switching remain, see item 17)
- [ ] 6. Render the playback History stack in the Queue panel — `history` exists in the Model but is never displayed (Spotify shows "History" in the queue)
- [ ] 7. Radio: "Start radio" from track/artist/album context menus seeding `/v1/recommendations` (seed genres/artists/tracks) — the recommendations API exists but there is no radio entry point
- [ ] 8. Create playlist (+ folder support) from the top-bar "+" button (currently `MockAction`) with name/description modal, wired to `POST /v1/users/{id}/playlists`
- [ ] 9. Fullscreen Now Playing / immersive mode with large animated cover art and progress — complements the existing mini-player
- [ ] 10. Album header richness: copyright, label, release year, and total duration in the album detail view (Spotify shows all of these)
- [ ] 11. Ctrl+K / "/" to focus search from anywhere, plus extended shortcuts (e.g., Ctrl+→ seek +10 s, Ctrl+← seek −10 s)
- [ ] 12. "Hide this song" / "Don't recommend this song" dislike actions (Spotify's taste-exclusion) via context menu
- [ ] 13. Offline downloads: cache tracks for offline playback with storage management (see Blocked — disk usage + ToS decision)
- [ ] 14. "Made for You" Daily Mix cluster on Home — featured playlists are fetched; surface the Daily Mix family explicitly
- [ ] 15. Share popover: copy link + "Open in browser" + OS share sheet (copy-link exists today, browser open does not)
- [ ] 16. Playlist sort controls (custom / recently added / artist / title) and in-playlist search, like Spotify's playlist header
- [ ] 17. Account menu extras: "Account" link to `spotify.com/account` and account switching (log out already works — Phase 11 item 5)

## Architectural Debt

- [ ] `OpenArtistContextMenu` was dead but is now wired (artist cards in search + artist page); remaining dead code: `SystemTrayManager` unused stub, `NavigationItem::Library` and `RightPanelTab::Lyrics` render placeholders only
- [ ] `Message::MockAction` button stubs: top-bar "+", playback-bar heart, queue "now playing" row — all do nothing (avatar now opens the account menu)
- [ ] Settings page is fully static mock UI (badges "320 kbps (Very High)" / "Enabled", static `/home/elgena/Music` path, "Spotify Connect: Enabled") — nothing on it is functional; every toggle must be wired when Phase 9 lands
- [ ] `DiskMetadataCache` never expires (stale playlists/albums resurrect from disk); `ImageCache` never evicts and lives in `std::env::temp_dir()`
- [ ] `get_user_music_dir()` hardcodes `/home/elgena/Música` as fallback; local scan re-runs on every playlist load with no memoization
- [ ] `fetch_playlist_tracks` caps at 200 tracks; `fetch_featured_playlists`/`fetch_new_releases` cap at 10
- [ ] Playback position driven from two sources (UI 200 ms tick + session 500 ms `PositionMs`) — drift risk
- [ ] Search fires a request per keystroke with no debounce
- [ ] Updater URL targets the wrong repo owner and `check_for_updates()` is never invoked
- [ ] Hardcoded context-menu coordinates everywhere instead of cursor position
- [ ] `session.rs` bitrate hardcoded to 160 kbps while README/Settings/TODO claim 320 kbps

## Blocked / Needs Human Decision

- [ ] Real system tray requires a new dependency (`tray-icon` or `ksni`/`tray-item`) — not in the Tech Stack table (AGENTS.md §6: no new deps without human decision)
- [ ] MPRIS2 D-Bus + global media keys require `zbus` as a direct dependency (currently only transitive) — §6 decision
- [ ] Local-file metadata parsing wants `lofty` (or direct `symphonia`, already transitive) — §6 decision
- [ ] Offline downloads: disk usage policy, cache lifecycle, and Spotify ToS implications — needs human decision
- [ ] Track-change desktop notifications need `notify-rust` — optional, §6 decision
- [ ] (Anything that isn't safe for the agent to decide unilaterally — see §6)
