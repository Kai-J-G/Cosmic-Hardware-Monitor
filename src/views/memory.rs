use cosmic::iced::{Alignment, Color, Length};
use cosmic::widget::{button, column, container, determinate_linear, row, text};
use cosmic::Element;

use crate::app::{ActiveTab, Message};
use crate::hardware::types::MemoryMetrics;
use crate::sparkline::Sparkline;

pub fn view_memory<'a>(
    memory: &'a MemoryMetrics,
    mem_history: &'a [f32],
    is_dark: bool,
) -> Element<'a, Message> {
    let sp = cosmic::theme::spacing();
    let emerald = Color::from_rgb(0.0, 0.90, 0.46);

    // 1. Navigation Breadcrumb Header
    let back_btn = button::text("← Overview")
        .padding([6, 12])
        .class(cosmic::theme::Button::Standard)
        .on_press(Message::SelectTab(ActiveTab::Overview));

    let title = text::title3("Memory & Swap Metrics").size(14);
    let nav_row = row![back_btn, cosmic::iced::widget::Space::new().width(Length::Fill), title]
        .align_y(Alignment::Center)
        .width(Length::Fill);

    // 2. In Use & Available Cards (Above the Graph)
    let in_use_sub = format!("{:.1}% of RAM", memory.percent);
    let avail_pct = if memory.total_bytes > 0 {
        (memory.available_bytes as f32 / memory.total_bytes as f32) * 100.0
    } else {
        0.0
    };
    let avail_sub = format!("{:.1}% free", avail_pct);

    let top_stat_card = |label: &'static str, value: String, sub: String| {
        let val_text = text::title1(value).size(22);
        let lbl_text = text::caption(label).size(11);
        let sub_text = text::caption(sub).size(10);
        let col = column![lbl_text, val_text, sub_text]
            .spacing(2)
            .align_x(Alignment::Center);

        container(col)
            .padding([10, 14])
            .width(Length::FillPortion(1))
            .height(Length::Fixed(82.0))
            .align_x(cosmic::iced::alignment::Horizontal::Center)
            .align_y(cosmic::iced::alignment::Vertical::Center)
            .class(cosmic::theme::Container::Custom(Box::new(move |_theme| {
                cosmic::iced::widget::container::Style {
                    background: Some(if is_dark {
                        Color::from_rgba(1.0, 1.0, 1.0, 0.04).into()
                    } else {
                        Color::from_rgba(0.0, 0.0, 0.0, 0.04).into()
                    }),
                    border: cosmic::iced::Border {
                        color: if is_dark {
                            Color::from_rgba(1.0, 1.0, 1.0, 0.08)
                        } else {
                            Color::from_rgba(0.0, 0.0, 0.0, 0.08)
                        },
                        width: 1.0,
                        radius: 12.0.into(),
                    },
                    ..Default::default()
                }
            })))
    };

    let in_use_and_available_row = row![
        top_stat_card("In Use", format_bytes(memory.used_bytes), in_use_sub),
        top_stat_card("Available", format_bytes(memory.available_bytes), avail_sub),
    ]
    .spacing(sp.space_s)
    .width(Length::Fill);

    // 3. Memory Usage History Graph Card
    let sparkline = Sparkline::new(mem_history, emerald).view(Length::Fill, Length::Fixed(80.0));
    let graph_label = text::caption("RAM Utilization History").size(11);
    let graph_content = column![graph_label, sparkline].spacing(sp.space_xs).width(Length::Fill);

    let graph_card = container(graph_content)
        .padding(sp.space_m)
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
                    radius: 14.0.into(),
                },
                ..Default::default()
            }
        })))
        .width(Length::Fill);

    // 4. RAM Overall Progress Bar
    let ram_bar = determinate_linear((memory.percent / 100.0).clamp(0.0, 1.0));
    let ram_summary = row![
        text::title3(format!("Physical Memory: {:.1}%", memory.percent)).size(13),
        cosmic::iced::widget::Space::new().width(Length::Fill),
        text::caption(format!(
            "{} in use / {} total",
            format_bytes(memory.used_bytes),
            format_bytes(memory.total_bytes)
        ))
        .size(11),
    ]
    .align_y(Alignment::Center)
    .width(Length::Fill);

    let ram_bar_box = column![ram_summary, ram_bar]
        .spacing(sp.space_xs)
        .width(Length::Fill);

    // 5. Additional Detailed Metrics: Committed, Cached, Swap Used, Swap Available
    let stat_card = |label: &'static str, value: String, sub: Option<String>| {
        let val_text = text::title3(value).size(15);
        let lbl_text = text::caption(label).size(11);
        let mut col = column![lbl_text, val_text].spacing(2);
        if let Some(s) = sub {
            col = col.push(text::caption(s).size(10));
        }

        container(col)
            .padding([8, 12])
            .width(Length::FillPortion(1))
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
            })))
    };

    let grid_row2 = row![
        stat_card("Committed", format_bytes(memory.committed_bytes), None),
        stat_card("Cached", format_bytes(memory.cached_bytes), None),
    ]
    .spacing(sp.space_s)
    .width(Length::Fill);

    let swap_pct = if memory.swap_total_bytes > 0 {
        (memory.swap_used_bytes as f32 / memory.swap_total_bytes as f32) * 100.0
    } else {
        0.0
    };
    let swap_sub = format!("{:.1}% used", swap_pct);

    let grid_row3 = row![
        stat_card("Swap Used", format_bytes(memory.swap_used_bytes), Some(swap_sub)),
        stat_card("Swap Available", format_bytes(memory.swap_avail_bytes), Some(format!("of {}", format_bytes(memory.swap_total_bytes)))),
    ]
    .spacing(sp.space_s)
    .width(Length::Fill);

    let metrics_col = column![grid_row2, grid_row3]
        .spacing(sp.space_s)
        .width(Length::Fill);

    column![nav_row, in_use_and_available_row, graph_card, ram_bar_box, metrics_col]
        .spacing(sp.space_m)
        .width(Length::Fill)
        .into()
}

fn format_bytes(bytes: u64) -> String {
    let mb = bytes as f64 / (1024.0 * 1024.0);
    if mb >= 1024.0 {
        format!("{:.2} GB", mb / 1024.0)
    } else {
        format!("{:.0} MB", mb)
    }
}
