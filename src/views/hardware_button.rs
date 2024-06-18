use iced::widget::{Button, Text};
use iced::{Color, Element};

use crate::custom_widgets::button_style::ButtonStyle;
use crate::{Message, Piggui, ToastMessage};

#[must_use]
pub fn hw_description(app: &Piggui) -> String {
    if let Some(hardware_description) = &app.hardware_description {
        format!(
            "Hardware: {}\nRevision: {}\nSerial: {}\nModel: {}",
            hardware_description.details.hardware,
            hardware_description.details.revision,
            hardware_description.details.serial,
            hardware_description.details.model,
        )
    } else {
        "No Hardware connected".to_string()
    }
}

/// Create the view that represents the clickable button that shows what hardware is connected
pub fn view(app: &Piggui) -> Element<Message> {
    let hw_text = if let Some(hardware_description) = &app.hardware_description {
        hardware_description.details.model.clone()
    } else {
        "No Hardware".to_string()
    };

    let about_button_style = ButtonStyle {
        bg_color: Color::TRANSPARENT,
        text_color: Color::WHITE,
        hovered_bg_color: Color::TRANSPARENT,
        hovered_text_color: Color::new(0.7, 0.7, 0.7, 1.0),
        border_radius: 4.0,
    };
    let add_toast_button = Button::new(Text::new(hw_text))
        .on_press(if !app.show_toast {
            // Add a new toast if `show_toast` is false
            Message::Toast(ToastMessage::HardwareDetailsToast)
        } else {
            // Close the existing toast if `show_toast` is true
            let index = app.toasts.len() - 1;
            Message::Toast(ToastMessage::Close(index))
        })
        .clip(true)
        .height(iced::Length::Shrink)
        .style(about_button_style.get_button_style());

    add_toast_button.into()
}
