//! A text-input test case for validating IME composition, Cyrillic
//! input, emoji ZWJ sequences, and copy/paste.
//!
//! This exists specifically to support the manual Android test
//! described in `docs/ARCHITECTURE.md`: `iced`'s own Android support
//! PR documents that the on-screen keyboard does not appear when a
//! text input is focused
//! (<https://github.com/iced-rs/iced/pull/3245>). This screen gives a
//! human tester a concrete input + a live breakdown of what was
//! actually received, to confirm or refute that on real hardware.

use iced::widget::{column, container, text, text_input};
use iced::{Element, Fill};
use unicode_segmentation::UnicodeSegmentation;

/// State owned by the IME test screen: just the current input value.
#[derive(Debug, Clone, Default)]
pub struct State {
    value: String,
}

/// Messages produced by the IME test screen.
#[derive(Debug, Clone)]
pub enum Message {
    /// The text input's value changed, via typing, IME commit, or paste.
    Changed(String),
}

impl State {
    /// Applies a [`Message`] to this screen's state.
    pub fn update(&mut self, message: Message) {
        match message {
            Message::Changed(value) => self.value = value,
        }
    }

    /// Builds the IME test view: a text input plus a live breakdown of
    /// what was actually received, so a human tester can confirm
    /// Cyrillic and multi-codepoint emoji were not corrupted.
    pub fn view(&self) -> Element<'_, Message> {
        let input = text_input(
            "Кириллица, эмодзи 👨‍👩‍👧‍👦 или ZWJ-последовательность…",
            &self.value,
        )
        .on_input(Message::Changed)
        .on_paste(Message::Changed)
        .size(20)
        .padding(12)
        .width(Fill);

        let breakdown = breakdown(&self.value);

        let report = column![
            text(format!("Байт (UTF-8): {}", breakdown.byte_count)),
            text(format!(
                "Unicode scalar values (`chars()`): {}",
                breakdown.scalar_count
            )),
            text(format!(
                "Grapheme clusters (то, что видит пользователь): {}",
                breakdown.grapheme_count
            )),
            text(
                "Проверка: эмодзи-семья 👨‍👩‍👧‍👦 должна дать 1 grapheme cluster \
                 при нескольких Unicode scalar values."
            ),
        ]
        .spacing(6);

        column![
            text("IME / Composition Test").size(24),
            input,
            container(report).padding(12),
        ]
        .spacing(16)
        .padding(24)
        .into()
    }
}

/// A breakdown of `value` at three different granularities, used to
/// visually prove emoji ZWJ sequences and Cyrillic text were not
/// corrupted in transit through the IME.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Breakdown {
    byte_count: usize,
    scalar_count: usize,
    grapheme_count: usize,
}

/// Computes the [`Breakdown`] of `value`, kept as a pure function so
/// it is unit-testable without a running `iced` event loop.
fn breakdown(value: &str) -> Breakdown {
    Breakdown {
        byte_count: value.len(),
        scalar_count: value.chars().count(),
        grapheme_count: value.graphemes(true).count(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cyrillic_text_is_not_corrupted() {
        let breakdown = breakdown("привет");
        assert_eq!(breakdown.scalar_count, 6);
        assert_eq!(breakdown.grapheme_count, 6);
        assert_eq!(breakdown.byte_count, "привет".len());
    }

    #[test]
    fn family_emoji_zwj_sequence_is_one_grapheme_cluster() {
        // 👨‍👩‍👧‍👦 = 4 emoji + 3 ZWJ joiners = 7 Unicode scalar values,
        // but a human perceives (and should be able to delete) it as
        // exactly one grapheme cluster.
        let breakdown = breakdown("👨\u{200d}👩\u{200d}👧\u{200d}👦");
        assert_eq!(breakdown.scalar_count, 7);
        assert_eq!(breakdown.grapheme_count, 1);
    }

    #[test]
    fn changed_message_replaces_the_value() {
        let mut state = State::default();
        state.update(Message::Changed("привет".to_owned()));
        assert_eq!(state.value, "привет");
    }
}
