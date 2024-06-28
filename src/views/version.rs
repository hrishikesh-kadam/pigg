use crate::styles::button_style::ButtonStyle;
use crate::{Message, ToastMessage};
use iced::widget::{Button, Text};
use iced::{Color, Element};

#[cfg(not(target_arch = "wasm32"))]
const VERSION: &str = env!("CARGO_PKG_VERSION");
#[cfg(not(target_arch = "wasm32"))]
const BIN_NAME: &str = env!("CARGO_BIN_NAME");
#[cfg(not(target_arch = "wasm32"))]
const PKG_NAME: &str = env!("CARGO_PKG_NAME");
#[cfg(not(target_arch = "wasm32"))]
const LICENSE: &str = env!("CARGO_PKG_LICENSE");
#[cfg(not(target_arch = "wasm32"))]
const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");

#[cfg(target_arch = "wasm32")]
const VERSION: &str = "0.1.0"; // Replace with your actual version
#[cfg(target_arch = "wasm32")]
const BIN_NAME: &str = "piggui";
#[cfg(target_arch = "wasm32")]
const PKG_NAME: &str = "pigg";
#[cfg(target_arch = "wasm32")]
const LICENSE: &str = "Apache-2.0";
#[cfg(target_arch = "wasm32")]
const REPOSITORY: &str = "https://github.com/andrewdavidmackenzie/pigg/";
#[must_use]
pub fn version() -> String {
    format!(
        "{bin_name} {version}\n\
        Copyright (C) 2024 The {pkg_name} Developers \n\
        License {license}: <https://www.gnu.org/licenses/{license_lower}.html>\n\
        This is free software: you are free to change and redistribute it.\n\
        There is NO WARRANTY, to the extent permitted by law.\n\
        \n\
        Written by the {pkg_name} Contributors.\n\
        Full source available at: {repository}",
        bin_name = BIN_NAME,
        pkg_name = PKG_NAME,
        version = VERSION,
        license = LICENSE,
        license_lower = LICENSE.to_lowercase(),
        repository = REPOSITORY,
    )
}

pub fn version_button() -> Element<'static, Message> {
    let version_text = Text::new(version().lines().next().unwrap_or_default().to_string());
    let about_button_style = ButtonStyle {
        bg_color: Color::TRANSPARENT,
        text_color: Color::new(0.7, 0.7, 0.7, 1.0),
        hovered_bg_color: Color::TRANSPARENT,
        hovered_text_color: Color::WHITE,
        border_radius: 4.0,
    };
    Button::new(version_text)
        .on_press(Message::Toast(ToastMessage::VersionToast))
        .clip(true)
        .height(iced::Length::Shrink)
        .style(about_button_style.get_button_style())
        .into()
}
