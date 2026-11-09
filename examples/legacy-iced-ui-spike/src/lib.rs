//! Phase 1 UI spike for the cross-platform Matrix client.
//!
//! This crate is deliberately **not** wired to Matrix at all yet — no
//! `matrix-sdk`, no network calls. Its only job is to prove out `iced`
//! as the UI toolkit before any client logic is built on top of it: a
//! sidebar + content shell, a 10,000+ row virtualized list (scroll
//! performance), and an IME/composition/paste test case (Cyrillic and
//! emoji ZWJ sequence input), with particular scrutiny on Android — see
//! `docs/ARCHITECTURE.md` for why that platform needs its own scrutiny.

// On Android, `Cargo.toml` pulls in the `iced` fork from
// `iced-rs/iced`'s `master` branch under the dependency key
// `iced-android` (see `crates/app-ui/Cargo.toml` for why it can't just
// be named `iced`). This re-binds it to the plain name `iced` crate-wide
// for that target, so every module below — and `android.rs` — can keep
// writing ordinary `iced::...` paths regardless of platform.
#[cfg(target_os = "android")]
extern crate iced_android as iced;
#[cfg(target_os = "android")]
extern crate iced_winit_android as iced_winit;

mod ime_test;
mod sidebar;
mod virtual_list;

#[cfg(target_os = "android")]
mod android;

use iced::widget::{container, row};
use iced::{Element, Fill, Theme};

/// Which content screen the sidebar currently has selected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Destination {
    /// The 10,000+ row virtualized list performance spike.
    #[default]
    VirtualList,
    /// The IME / composition / clipboard test case.
    ImeTest,
}

/// Top-level application state.
#[derive(Debug, Default)]
pub struct State {
    destination: Destination,
    virtual_list: virtual_list::State,
    ime_test: ime_test::State,
}

/// Top-level application messages.
#[derive(Debug, Clone)]
pub enum Message {
    /// The sidebar selection changed.
    Navigate(Destination),
    /// A message for the virtualized list screen.
    VirtualList(virtual_list::Message),
    /// A message for the IME test screen.
    ImeTest(ime_test::Message),
}

fn update(state: &mut State, message: Message) {
    match message {
        Message::Navigate(destination) => state.destination = destination,
        Message::VirtualList(message) => state.virtual_list.update(message),
        Message::ImeTest(message) => state.ime_test.update(message),
    }
}

fn view(state: &State) -> Element<'_, Message> {
    let content: Element<'_, Message> = match state.destination {
        Destination::VirtualList => state.virtual_list.view().map(Message::VirtualList),
        Destination::ImeTest => state.ime_test.view().map(Message::ImeTest),
    };

    row![
        sidebar::view(state.destination),
        container(content).width(Fill).height(Fill).padding(16),
    ]
    .into()
}

/// Runs the application. Shared by the desktop binary
/// (`src/main.rs`) and, on Android, [`android::android_main`].
///
/// # Errors
///
/// Returns an error if the underlying `iced` runtime fails to start,
/// e.g. no suitable graphics adapter was found.
pub fn run() -> iced::Result {
    iced::application(State::default, update, view)
        .title("Matrix Client — UI Spike")
        .theme(Theme::Dark)
        .run()
}
