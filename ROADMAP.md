# Roadmap

*[Читать на русском](ROADMAP.ru.md)*

Scope for this repository is **desktop only: Windows + Linux**, no
Android/iOS/macOS. A phone client is planned as a **separate repository
with its own roadmap**, not a milestone in this one.

Each milestone below lists concrete tasks with acceptance criteria.
Don't start the next milestone's tasks until the current one's criteria
are actually met — avoid working on several milestones in parallel.

## Current state — verified 2026-11-09

**Backend** (`crates/matrix-client`, on `matrix-sdk`/`matrix-sdk-ui`
0.19.1):
- Login (username/password) and session restore, persisted to disk on
  the Flutter side.
- Sync via `SyncService`.
- Joined room list: snapshot + live updates.
- Top-level joined spaces: snapshot + live updates. A space's child
  rooms: snapshot only (no live updates yet).
- Per-room timeline: snapshot + live updates, plain-text send.
- Not built yet: replies, reactions, edits, search, read receipts,
  typing indicators, media (images/files — `matrix-storage` only has
  cache *retention policy*, no actual download/display path), E2EE
  verified against a pre-existing encrypted room, voice/video.

**UI** (Flutter, `flutter_rust_bridge`):
- Real dark theme matching the project's own Cinny-based design
  reference (`design/`), not Material defaults.
- Real 3-pane desktop layout: sidebar rail (spaces + nav) / room list /
  room view — not a placeholder.
- Only the Home tab and a selected space have real screens behind them.
  Direct Messages, Explore, Search, Inbox, and most of Settings are
  explicit "Not implemented yet" placeholders — intentional, not faked
  data, but also not built.
- Logout works.

**Platforms:**
- Linux desktop: built and run-tested repeatedly.
- Windows desktop: the Rust workspace is checked/tested on Windows in
  CI (`cargo check`/`clippy`/`test`), but the actual Flutter Windows
  **build has never been attempted**. Treat Windows as unverified until
  someone actually runs `flutter build windows`.

**Testing/CI:**
- 3 Rust tests (mock homeserver, no real network) + 1 Flutter widget
  test file.
- `.github/workflows/ci.yml` runs `cargo fmt`/`check`/`clippy`/`test` on
  Ubuntu+Windows, plus `cargo-deny`/`cargo-audit` — Rust-only, doesn't
  build or test the Flutter app itself.

**Suggested current build tag: `0.1.0-alpha`.** The defining deliverable
of the 0.1.0 milestone below (real 3-column desktop UI, no blue
defaults, wired to live data) is substantially met; the milestone's
remaining items (member drawer, settings/search dialogs, DM screen) are
listed as open tasks below rather than claimed as done.

---

## 0.0.1-alpha — done

- [x] Log in to a Matrix homeserver (username/password).
- [x] Start sync, confirm it actually runs against a real server.
- [x] Send a plain-text message and read it back.

## 0.1.0-alpha — UI shell (in progress)

**Goal:** a real, fully custom-styled desktop interface, not a
backend-logic demo.

- [x] 3-pane desktop layout (sidebar rail / room list / room view).
- [x] Custom dark theme, no Material blue defaults.
- [x] Real joined-room and joined-space navigation, live-updating.
- [x] Member list drawer for an open room.
- [ ] Settings dialog: at minimum profile display (avatar, display
  name, user ID) alongside the existing logout button.
  **Acceptance:** opening Settings shows the logged-in user's real
  profile data, not just the logout button.
- [ ] Direct Messages tab: list of real DM rooms (1:1 rooms, not
  flagged as spaces).
  **Acceptance:** a DM started from another client shows up in this
  tab with the other user's name.
- [ ] Windows desktop build actually attempted and confirmed working.
  **Acceptance:** `flutter build windows` succeeds and the built `.exe`
  launches, logs in, and sends a message on a real Windows machine (or
  CI runner).

## 0.2.0-alpha — real chat features

**Goal:** a timeline that behaves like a real chat client, not just a
scrollback of plain-text lines.

- [ ] Reply to a message.
  **Acceptance:** sending a reply shows the quoted original inline, and
  round-trips correctly when read by another client (e.g. Element).
- [ ] React to a message with an emoji.
  **Acceptance:** reactions appear under the message, aggregate
  correctly across users, and are visible in another client.
- [ ] Edit a sent message.
  **Acceptance:** an edited message shows "(edited)", and the edit
  history is at least retrievable (doesn't need a full UI for history
  yet).
- [ ] Search within the current room's timeline.
  **Acceptance:** typing a query highlights/jumps to matching messages
  already loaded in the open timeline (server-side search across all
  rooms can wait for a later pass).

## 0.3.0-alpha — spaces and roles

**Goal:** spaces and rooms become manageable, not just browsable.

- [x] List top-level joined spaces and a space's child rooms.
- [ ] Show a room member's role (admin/moderator/member) based on power
  levels.
  **Acceptance:** the member drawer shows a role label per member,
  matching the room's actual `m.room.power_levels` state.
- [ ] Gate admin-only actions (kick, ban, room settings edit) by the
  current user's own power level.
  **Acceptance:** a non-privileged user doesn't see/can't trigger
  actions they lack permission for; a privileged user can.
- [ ] Create a room or space from the UI.
  **Acceptance:** a room/space created in-app is immediately visible to
  another client joined to the same account.

## 0.5.0-beta — media

**Goal:** images and files actually work, not just text.

- [ ] Download and display images inline in the timeline.
  **Acceptance:** an image sent from another client renders inline,
  using `matrix-storage`'s cache (not re-downloaded on every scroll).
- [ ] Send an image/file from the UI.
  **Acceptance:** a file sent from this app opens correctly in another
  client.
- [ ] Media gallery view for a room.
  **Acceptance:** opening a room's gallery shows all its images in a
  grid, loaded from cache where available.

## 0.7.0-beta — voice

**Goal:** basic voice calls work without being unusable.

- [ ] 1:1 voice call via WebRTC (matrix-sdk's call support).
  **Acceptance:** a call connects, audio is heard both directions, and
  normal network conditions don't produce noticeable lag/dropouts.

## 0.9.0-rc — stabilization

**Goal:** confidence to call it done, on both real target platforms.

- [ ] Full manual click-through test pass on both Linux and Windows.
  **Acceptance:** a written test log covering login, sync, rooms,
  spaces, timeline, send, media, reactions/replies/edits, voice — on
  both OSes, no crashes.
- [ ] All known critical bugs from the above pass fixed.
- [ ] CI builds and smoke-tests the actual Flutter app (not just the
  Rust workspace) on both platforms.

## 1.0.0 — release

- [ ] UX polish pass against the `design/` reference, no known rough
  edges in the core flows.
- [ ] User-facing documentation (how to install, log in, basic usage).
- [ ] Android client: **not in this repository.** Tracked as a future,
  separate project with its own roadmap once this one reaches 1.0.
