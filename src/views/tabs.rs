use cosmic::iced::alignment::Horizontal;
use cosmic::iced::{Alignment, Color, Length};
use cosmic::widget::{button, container, row, text};
use cosmic::Element;

use crate::app::{ActiveTab, Message};

#[allow(dead_code)]
pub fn view_tabs<'a>(active_tab: ActiveTab, is_dark: bool) -> Element<'a, Message> {
    let sp = cosmic::theme::spacing();

    let tab_item = |tab: ActiveTab, label: &'static str| {
        let is_active = tab == active_tab;
        button::custom(text(label).size(11))
            .padding([5, 14])
            .class(if is_active {
                cosmic::theme::Button::Suggested
            } else {
                cosmic::theme::Button::Standard
            })
            .on_press(Message::SelectTab(tab))
    };

    let tabs_row = row![
        tab_item(ActiveTab::Overview, "Overview"),
        tab_item(ActiveTab::CpuDetail, "CPU"),
        tab_item(ActiveTab::GpuDetail, "GPU"),
        tab_item(ActiveTab::MemoryDetail, "Memory"),
        tab_item(ActiveTab::StorageDetail, "Storage"),
    ]
    .spacing(sp.space_xs)
    .align_y(Alignment::Center);

    let wrapped = container(tabs_row)
        .padding(3)
        .class(cosmic::theme::Container::Custom(Box::new(move |_theme| {
            cosmic::iced::widget::container::Style {
                background: Some(if is_dark {
                    Color::from_rgba(1.0, 1.0, 1.0, 0.03).into()
                } else {
                    Color::from_rgba(0.0, 0.0, 0.0, 0.03).into()
                }),
                border: cosmic::iced::Border {
                    color: if is_dark {
                        Color::from_rgba(1.0, 1.0, 1.0, 0.06)
                    } else {
                        Color::from_rgba(0.0, 0.0, 0.0, 0.08)
                    },
                    width: 1.0,
                    radius: 10.0.into(),
                },
                ..Default::default()
            }
        })));

    container(wrapped)
        .align_x(Horizontal::Center)
        .width(Length::Fill)
        .into()
}
