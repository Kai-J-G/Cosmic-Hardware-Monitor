use cosmic::iced::{Alignment, Length};
use cosmic::widget::{button, container, icon, row, text};
use cosmic::Element;

use crate::app::{ActiveTab, Message};

pub fn view_header<'a>(active_tab: ActiveTab) -> Element<'a, Message> {
    // 1. Left: Spacer to balance the right settings button and keep title centered
    let left_spacer = cosmic::iced::widget::Space::new().width(Length::Fixed(40.0));

    // 2. Center: Title
    let title = text::title3("Hardware Monitor").size(15);

    let center_group = container(title)
        .align_x(cosmic::iced::alignment::Horizontal::Center)
        .width(Length::Fill);

    // 3. Right: Settings gear icon button
    let target_tab = if active_tab == ActiveTab::Settings {
        ActiveTab::Overview
    } else {
        ActiveTab::Settings
    };

    let settings_btn = button::icon(icon::from_name("emblem-system-symbolic").size(16))
        .padding(8)
        .class(if active_tab == ActiveTab::Settings {
            cosmic::theme::Button::Suggested
        } else {
            cosmic::theme::Button::Icon
        })
        .on_press(Message::SelectTab(target_tab));

    let right_group = container(settings_btn)
        .align_x(cosmic::iced::alignment::Horizontal::Right)
        .width(Length::Fixed(40.0));

    row![left_spacer, center_group, right_group]
        .align_y(Alignment::Center)
        .width(Length::Fill)
        .into()
}
