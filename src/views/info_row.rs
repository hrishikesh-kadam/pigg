use crate::connect_dialog_handler::ConnectDialogMessage;
use crate::styles::background::SetAppearance;
use crate::styles::button_style::ButtonStyle;
use crate::views::hardware_view::{HardwareTarget, HardwareView};
use crate::views::message_row::{MessageMessage, MessageRow, MessageRowMessage};
use crate::views::version::version_button;
use crate::views::{hardware_button, unsaved_status};
use crate::Message;
use iced::widget::{container, Button, Row, Text};
use iced::{Color, Command, Element, Length};
use iced_aw::menu;
use iced_aw::menu::{Item, Menu, MenuBar, StyleSheet};
use iced_aw::style::MenuBarStyle;
use iced_futures::core::Background;
use iced_futures::Subscription;

const MENU_WIDTH: f32 = 200.0;

const ENABLED_MENU_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    bg_color: Color::TRANSPARENT,
    text_color: Color::WHITE,
    hovered_bg_color: Color::TRANSPARENT,
    hovered_text_color: Color::WHITE,
    border_radius: 4.0,
};

const DISABLED_MENU_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    bg_color: Color::TRANSPARENT,
    text_color: Color::from_rgb(0.5, 0.5, 0.5), // Medium grey text color
    hovered_bg_color: Color::from_rgb(0.2, 0.2, 0.2),
    hovered_text_color: Color::from_rgb(0.5, 0.5, 0.5),
    border_radius: 4.0,
};

pub struct InfoRow {
    message_row: MessageRow,
}

impl InfoRow {
    /// Create a new InfoRow
    pub fn new() -> Self {
        Self {
            message_row: MessageRow::new(),
        }
    }

    /// Add a message to the queue of messages to display in the message_row
    pub fn add_info_message(&mut self, msg: MessageMessage) {
        self.message_row.add_message(msg);
    }

    /// Update state based on [MessageRowMessage] messages received
    pub fn update(&mut self, message: MessageRowMessage) -> Command<Message> {
        self.message_row.update(message)
    }

    /// Create the view that represents the info row at the bottom of the window
    pub fn view<'a>(
        &'a self,
        unsaved_changes: bool,
        hardware_view: &'a HardwareView,
        hardware_target: &HardwareTarget,
    ) -> Element<'a, Message> {
        let menu_bar_button_style = ButtonStyle {
            bg_color: Color::TRANSPARENT,
            text_color: Color::new(0.7, 0.7, 0.7, 1.0),
            hovered_bg_color: Color::TRANSPARENT,
            hovered_text_color: Color::WHITE,
            border_radius: 4.0,
        };

        let model = match hardware_view.hw_model() {
            None => "No Hardware connected".to_string(),
            Some(model) => match hardware_target {
                HardwareTarget::Local => format!("{}@Local", model),
                HardwareTarget::Remote(_, _) => format!("{}@Remote", model),
                HardwareTarget::NoHW => "No Hardware connected".to_string(),
            },
        };

        // Conditionally render menu items based on hardware features
        let mut menu_items: Vec<Item<'a, Message, _, _>> = vec![];

        #[cfg(any(feature = "pi_hw", feature = "fake_hw"))]
        menu_items.push(Item::new(
            Button::new("Use local GPIO")
                .on_press(Message::ConnectDialog(ConnectDialogMessage::ConnectLocal))
                .style(ENABLED_MENU_BUTTON_STYLE.get_button_style())
                .width(Length::Fill),
        ));

        menu_items.push(Item::new(
            Button::new("Connect to remote Pi...")
                .width(Length::Fill)
                .on_press(Message::ConnectDialog(
                    ConnectDialogMessage::ShowConnectDialog,
                ))
                .style(ENABLED_MENU_BUTTON_STYLE.get_button_style()),
        ));

        #[cfg(feature = "discovery")]
        menu_items.push(Item::new(
            Button::new("Search for Pi's on local network...")
                .width(Length::Fill)
                .style(ENABLED_MENU_BUTTON_STYLE.get_button_style()),
        ));

        menu_items.push(Item::new(hardware_button::view()));

        let hardware_root = Item::with_menu(
            Button::new(Text::new(model))
                .style(menu_bar_button_style.get_button_style())
                .on_press(Message::MenuBarButtonClicked),
            Menu::new(menu_items)
                .width(MENU_WIDTH)
                .spacing(2.0)
                .offset(10.0),
        );

        let mb = MenuBar::new(vec![hardware_root]).style(|theme: &iced::Theme| menu::Appearance {
            bar_background: Background::Color(Color::TRANSPARENT),
            menu_shadow: iced::Shadow {
                color: Color::BLACK,
                offset: iced::Vector::new(1.0, 1.0),
                blur_radius: 10f32,
            },
            menu_background_expand: iced::Padding::from([5, 5]),
            ..theme.appearance(&MenuBarStyle::Default)
        });

        container(
            Row::new()
                .push(version_button())
                .push(mb)
                .push(unsaved_status::view(unsaved_changes))
                .push(iced::widget::Space::with_width(Length::Fill)) // This takes up remaining space
                .push(self.message_row.view().map(Message::InfoRow))
                .spacing(20.0)
                .padding([0.0, 0.0, 0.0, 0.0]),
        )
        .set_background(Color::from_rgb8(45, 45, 45))
        .into()
    }

    pub fn subscription(&self) -> Subscription<MessageRowMessage> {
        self.message_row.subscription()
    }
}
