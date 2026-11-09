//! A 10,000+ row virtualized list, used to validate that `iced`'s
//! `scrollable` plus manual windowing performs acceptably at that
//! scale (target: >= 60 FPS on mid-range desktop hardware; see
//! `docs/ARCHITECTURE.md` for the measurement procedure).
//!
//! `iced` 0.14 has no built-in equivalent to a Compose `LazyColumn`:
//! its `scrollable` widget always lays out whatever `Element` it is
//! given. This module implements virtualization manually — the
//! pattern used throughout the `iced` community — by tracking the
//! scroll offset via `Scrollable::on_scroll`, computing which row
//! indices are visible for a fixed row height, and only building
//! `Element`s for that window. Blank [`Space`] widgets above and below
//! the window are sized to match the un-rendered rows, so the
//! scrollbar still reflects the full, true content length rather than
//! only the rendered window.
//!
//! The very first frame (before any scroll event has fired) still
//! needs to know how tall the viewport is to pick the right window;
//! this is solved with [`responsive`], which hands the view closure
//! the exact available [`Size`] at layout time.

use iced::widget::scrollable::Viewport;
use iced::widget::{Space, column, container, responsive, scrollable, text};
use iced::{Element, Fill, Length};

/// Number of synthetic rows rendered by the spike — matches the
/// Phase 1 acceptance criterion of validating scroll performance at
/// 10,000+ items.
pub const ITEM_COUNT: usize = 12_000;

/// Fixed height of a single row, in logical pixels.
const ROW_HEIGHT: f32 = 32.0;

/// Extra rows rendered above/below the visible window so a fast flick
/// does not momentarily reveal blank space before the next frame's
/// window recalculation catches up.
const OVERSCAN_ROWS: usize = 4;

/// State owned by the virtualized list screen: just the current
/// vertical scroll offset, in logical pixels.
#[derive(Debug, Clone, Copy, Default)]
pub struct State {
    scroll_offset_y: f32,
}

/// Messages produced by the virtualized list screen.
#[derive(Debug, Clone, Copy)]
pub enum Message {
    /// The user scrolled; carries the new absolute vertical offset.
    Scrolled(f32),
}

/// Computes the half-open range `[first, last)` of row indices that
/// must be rendered for a viewport `viewport_height` pixels tall,
/// scrolled `offset_y` pixels down, given the fixed [`ROW_HEIGHT`] and
/// [`OVERSCAN_ROWS`] — the core virtualization decision, kept as a
/// pure function so it is unit-testable without a running `iced`
/// event loop.
fn visible_window(offset_y: f32, viewport_height: f32) -> (usize, usize) {
    let visible_rows = (viewport_height / ROW_HEIGHT).ceil() as usize + OVERSCAN_ROWS * 2;

    let first_visible = ((offset_y / ROW_HEIGHT).floor() as usize)
        .saturating_sub(OVERSCAN_ROWS)
        .min(ITEM_COUNT);
    let last_visible = (first_visible + visible_rows).min(ITEM_COUNT);

    (first_visible, last_visible)
}

impl State {
    /// Applies a [`Message`] to this screen's state.
    pub fn update(&mut self, message: Message) {
        match message {
            Message::Scrolled(offset_y) => self.scroll_offset_y = offset_y,
        }
    }

    /// Builds the virtualized list view.
    pub fn view(&self) -> Element<'_, Message> {
        let offset_y = self.scroll_offset_y;

        responsive(move |size| {
            let (first_visible, last_visible) = visible_window(offset_y, size.height);

            let top_spacer = first_visible as f32 * ROW_HEIGHT;
            let bottom_spacer = (ITEM_COUNT - last_visible) as f32 * ROW_HEIGHT;

            let rows = (first_visible..last_visible).map(|index| {
                container(text(format!(
                    "Сообщение #{index} — тестовая строка виртуализированного списка"
                )))
                .height(Length::Fixed(ROW_HEIGHT))
                .padding(4)
                .into()
            });

            let content = column(
                std::iter::once(Space::new().width(Fill).height(Length::Fixed(top_spacer)).into())
                    .chain(rows)
                    .chain(std::iter::once(
                        Space::new()
                            .width(Fill)
                            .height(Length::Fixed(bottom_spacer))
                            .into(),
                    )),
            );

            scrollable(content)
                .on_scroll(|viewport: Viewport| Message::Scrolled(viewport.absolute_offset().y))
                .height(Fill)
                .into()
        })
        .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_state_starts_scrolled_to_the_top() {
        assert_eq!(State::default().scroll_offset_y, 0.0);
    }

    #[test]
    fn scrolled_message_updates_the_offset() {
        let mut state = State::default();
        state.update(Message::Scrolled(1_234.5));
        assert_eq!(state.scroll_offset_y, 1_234.5);
    }

    #[test]
    fn window_at_the_top_starts_at_row_zero() {
        let (first, last) = visible_window(0.0, 600.0);
        assert_eq!(first, 0);
        // `first` is clamped to 0 (can't scroll above the top), but the
        // window height itself always includes the full 2 * OVERSCAN_ROWS,
        // regardless of clamping, so the edge window is simply larger.
        assert_eq!(last, (600.0 / ROW_HEIGHT).ceil() as usize + OVERSCAN_ROWS * 2);
    }

    #[test]
    fn window_scrolled_past_the_top_is_offset_minus_overscan() {
        // Scrolled 10 rows down: naive first-visible would be row 10,
        // but overscan pulls it back by OVERSCAN_ROWS.
        let (first, _last) = visible_window(10.0 * ROW_HEIGHT, 600.0);
        assert_eq!(first, 10 - OVERSCAN_ROWS);
    }

    #[test]
    fn window_never_exceeds_item_count_even_near_the_end() {
        let near_end_offset = (ITEM_COUNT - 5) as f32 * ROW_HEIGHT;
        let (first, last) = visible_window(near_end_offset, 600.0);
        assert!(first <= ITEM_COUNT);
        assert!(last <= ITEM_COUNT);
    }
}
