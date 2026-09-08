use cosmic::iced::{Alignment, Color, Length};
use cosmic::widget::{button, column, container, icon, row, scrollable, text};
use cosmic::Element;

use crate::app::{ActiveTab, Message};
use crate::hardware::types::FanInfo;

#[allow(dead_code)]
pub fn view_fans<'a>(fans: &'a [FanInfo]) -> Element<'a, Message> {
    let sp = cosmic::theme::spacing();

    let back_btn = button::text("← Overview")
        .padding([6, 12])
        .class(cosmic::theme::Button::Standard)
        .on_press(Message::SelectTab(ActiveTab::Overview));

    let title = text::title3("Cooling Fans & Tachometers").size(14);
    let nav_row = row![back_btn, cosmic::iced::widget::Space::new().width(Length::Fill), title]
        .align_y(Alignment::Center)
        .width(Length::Fill);

    if fans.is_empty() {
        let msg = container(text::body(
            "No controllable fan tachometers detected.",
        ))
        .padding(sp.space_m)
        .class(cosmic::theme::Container::Custom(Box::new(|_theme| {
            cosmic::iced::widget::container::Style {
                background: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.03).into()),
                border: cosmic::iced::Border {
                    color: Color::from_rgba(1.0, 1.0, 1.0, 0.06),
                    width: 1.0,
                    radius: 12.0.into(),
                },
                ..Default::default()
            }
        })))
        .width(Length::Fill);

        return column![nav_row, msg].spacing(sp.space_m).width(Length::Fill).into();
    }

    let mut list = column![].spacing(sp.space_xs).width(Length::Fill);

    for fan in fans {
        let is_active = fan.is_active;
        let icon_name = if is_active {
            "fan-symbolic"
        } else {
            "media-playback-stop-symbolic"
        };

        let fan_icon = icon::from_name(icon_name).size(18);

        let name_col = column![
            text::body(&fan.name).size(13),
            text::caption(&fan.chip).size(11)
        ]
        .spacing(1);

        let rpm_text = text::title3(format!("{} RPM", fan.rpm)).size(15);

        let status_badge = container(
            text::caption(if is_active { "ACTIVE" } else { "IDLE / STOP" }).size(10),
        )
        .padding([2, 6])
        .class(cosmic::theme::Container::Custom(Box::new(move |_| {
            cosmic::iced::widget::container::Style {
                background: Some(
                    if is_active {
                        Color::from_rgba(0.3, 0.85, 0.4, 0.15)
                    } else {
                        Color::from_rgba(1.0, 1.0, 1.0, 0.08)
                    }
                    .into(),
                ),
                border: cosmic::iced::border::rounded(6.0),
                ..Default::default()
            }
        })));

        let right_group = row![rpm_text, status_badge]
            .spacing(sp.space_xs)
            .align_y(Alignment::Center);

        let spacer = cosmic::iced::widget::Space::new().width(Length::Fill);
        let row_item = row![fan_icon, name_col, spacer, right_group]
            .spacing(sp.space_s)
            .align_y(Alignment::Center)
            .width(Length::Fill);

        let card = container(row_item)
            .padding([sp.space_s, sp.space_m])
            .class(cosmic::theme::Container::Custom(Box::new(|_theme| {
                cosmic::iced::widget::container::Style {
                    background: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.03).into()),
                    border: cosmic::iced::Border {
                        color: Color::from_rgba(1.0, 1.0, 1.0, 0.06),
                        width: 1.0,
                        radius: 12.0.into(),
                    },
                    ..Default::default()
                }
            })))
            .width(Length::Fill);

        list = list.push(card);
    }

    let scrollable_list = scrollable(list)
        .height(Length::Fixed(240.0))
        .width(Length::Fill);

    column![nav_row, scrollable_list]
        .spacing(sp.space_m)
        .width(Length::Fill)
        .into()
}
