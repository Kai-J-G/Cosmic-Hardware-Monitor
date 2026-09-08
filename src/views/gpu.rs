use cosmic::iced::{Alignment, Color, Length};
use cosmic::widget::{button, column, container, determinate_linear, row, text};
use cosmic::Element;

use crate::app::{ActiveTab, Message};
use crate::hardware::types::{GpuInfo, TemperatureUnit, ThermalStatus};
use crate::sparkline::Sparkline;

pub fn view_gpu<'a>(
    gpu: Option<&'a GpuInfo>,
    gpu_history: &'a [f32],
    unit: TemperatureUnit,
    is_dark: bool,
) -> Element<'a, Message> {
    let sp = cosmic::theme::spacing();

    let Some(gpu) = gpu else {
        let back_btn = button::text("← Overview")
            .padding([6, 12])
            .class(cosmic::theme::Button::Standard)
            .on_press(Message::SelectTab(ActiveTab::Overview));

        let msg = text::body("No dedicated GPU detected or GPU metrics unavailable.");
        return column![back_btn, container(msg).padding(sp.space_m).width(Length::Fill)]
            .spacing(sp.space_m)
            .width(Length::Fill)
            .into();
    };

    let status = ThermalStatus::from_celsius(gpu.edge_temp);
    let [er, eg, eb] = status.rgb();
    let state_color = Color::from_rgb(er, eg, eb);

    // 1. Navigation Breadcrumb Header
    let back_btn = button::text("← Overview")
        .padding([6, 12])
        .class(cosmic::theme::Button::Standard)
        .on_press(Message::SelectTab(ActiveTab::Overview));

    let title = text::title3(&gpu.name).size(14);
    let nav_row = row![back_btn, cosmic::iced::widget::Space::new().width(Length::Fill), title]
        .spacing(sp.space_s)
        .align_y(Alignment::Center)
        .width(Length::Fill);

    // 2. The 4 Required Metrics: Utilization, Clock Speed, Power Draw, Temperature
    let stat_card = |label: &'static str, value: String, sub: Option<String>, accent: Option<Color>| {
        let val_text = text::title1(value).size(22);
        let lbl_text = text::caption(label).size(11);

        let sub_widget = if let Some(s) = sub {
            text::caption(s).size(10)
        } else {
            text(" ").size(10)
        };

        let col = column![lbl_text, val_text, sub_widget]
            .spacing(2)
            .align_x(Alignment::Center);

        container(col)
            .padding([10, 12])
            .width(Length::FillPortion(1))
            .height(Length::Fixed(84.0))
            .align_x(cosmic::iced::alignment::Horizontal::Center)
            .align_y(cosmic::iced::alignment::Vertical::Center)
            .class(cosmic::theme::Container::Custom(Box::new(move |_theme| {
                cosmic::iced::widget::container::Style {
                    background: Some(if let Some(acc) = accent {
                        Color::from_rgba(acc.r, acc.g, acc.b, if is_dark { 0.12 } else { 0.08 }).into()
                    } else if is_dark {
                        Color::from_rgba(1.0, 1.0, 1.0, 0.035).into()
                    } else {
                        Color::from_rgba(0.0, 0.0, 0.0, 0.035).into()
                    }),
                    border: cosmic::iced::Border {
                        color: if let Some(acc) = accent {
                            Color::from_rgba(acc.r, acc.g, acc.b, if is_dark { 0.35 } else { 0.25 })
                        } else if is_dark {
                            Color::from_rgba(1.0, 1.0, 1.0, 0.07)
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

    let util_str = format!("{}%", gpu.utilization_percent);
    let clock_str = match gpu.clock_mhz {
        Some(clk) => format!("{} MHz", clk),
        None => "N/A".to_string(),
    };
    let power_str = match gpu.power_draw_watts {
        Some(p) => format!("{:.1} W", p),
        None => "N/A".to_string(),
    };
    let temp_str = unit.format_temp(gpu.edge_temp);
    let temp_sub = match gpu.junction_temp {
        Some(j) => Some(format!("Junction: {}", unit.format_temp_short(j))),
        None => None,
    };

    let metrics_row = row![
        stat_card("Utilization", util_str, None, None),
        stat_card("Clock Speed", clock_str, None, None),
        stat_card("Power Draw", power_str, None, None),
        stat_card("Temperature", temp_str, temp_sub, Some(state_color)),
    ]
    .spacing(sp.space_s)
    .width(Length::Fill);

    // 3. GPU Temperature Curve Card
    let sparkline = Sparkline::new(gpu_history, state_color).view(Length::Fill, Length::Fixed(76.0));
    let graph_label = text::caption("GPU Temperature Curve").size(11);
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
                    radius: 12.0.into(),
                },
                ..Default::default()
            }
        })))
        .width(Length::Fill);

    // 4. VRAM & Hardware Status
    let vram_used_gb = gpu.vram_used_bytes as f64 / 1_073_741_824.0;
    let vram_total_gb = gpu.vram_total_bytes as f64 / 1_073_741_824.0;
    let vram_pct = if gpu.vram_total_bytes > 0 {
        (gpu.vram_used_bytes as f32 / gpu.vram_total_bytes as f32) * 100.0
    } else {
        0.0
    };

    let vram_bar = determinate_linear((vram_pct / 100.0).clamp(0.0, 1.0));
    let vram_header = row![
        text::title3(format!("VRAM Usage: {:.1}%", vram_pct)).size(13),
        cosmic::iced::widget::Space::new().width(Length::Fill),
        text::caption(format!("{:.2} GB / {:.2} GB", vram_used_gb, vram_total_gb)).size(11),
    ]
    .align_y(Alignment::Center)
    .width(Length::Fill);

    let vram_card = container(column![vram_header, vram_bar].spacing(sp.space_xs).width(Length::Fill))
        .padding(sp.space_m)
        .class(cosmic::theme::Container::Custom(Box::new(move |_theme| {
            cosmic::iced::widget::container::Style {
                background: Some(if is_dark {
                    Color::from_rgba(1.0, 1.0, 1.0, 0.035).into()
                } else {
                    Color::from_rgba(0.0, 0.0, 0.0, 0.035).into()
                }),
                border: cosmic::iced::Border {
                    color: if is_dark {
                        Color::from_rgba(1.0, 1.0, 1.0, 0.06)
                    } else {
                        Color::from_rgba(0.0, 0.0, 0.0, 0.08)
                    },
                    width: 1.0,
                    radius: 12.0.into(),
                },
                ..Default::default()
            }
        })))
        .width(Length::Fill);

    column![nav_row, metrics_row, graph_card, vram_card]
        .spacing(sp.space_s)
        .width(Length::Fill)
        .into()
}
