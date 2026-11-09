//! The application's left-hand navigation sidebar.

use crate::{Destination, Message};
use iced::widget::{button, column, container, text};
use iced::{Element, Fill, Length};

/// Builds the sidebar, highlighting `current` as the active destination.
///
/// Also exercises mouse *and* touch input identically: `iced`'s
/// `Button` widget reacts to both through the same `winit` pointer
/// events, so no separate touch handling is needed here.
pub fn view(current: Destination) -> Element<'static, Message> {
    let entry = |destination: Destination| -> Element<'static, Message> {
        let is_active = destination == current;

        button(text(label(destination)))
            .width(Fill)
            .on_press(Message::Navigate(destination))
            .style(move |theme, status| {
                if is_active {
                    button::primary(theme, status)
                } else {
                    button::text(theme, status)
                }
            })
            .into()
    };

    container(
        column![entry(Destination::VirtualList), entry(Destination::ImeTest)]
            .spacing(4)
            .padding(12)
            .width(Length::Fixed(220.0)),
    )
    .height(Fill)
    .into()
}

/// The label shown for a [`Destination`] in the sidebar.
fn label(destination: Destination) -> &'static str {
    match destination {
        Destination::VirtualList => "Виртуализированный список",
        Destination::ImeTest => "IME / ввод текста",
    }
}
