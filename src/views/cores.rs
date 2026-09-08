use cosmic::iced::{Alignment, Color, Length};
use cosmic::widget::{button, column, container, mouse_area, row, text};
use cosmic::Element;

use super::vertical_bar::VerticalBar;
use crate::app::{ActiveTab, Message};
use crate::hardware::types::{CpuInfo, TemperatureUnit, ThermalStatus};
use crate::sparkline::Sparkline;

pub fn view_cores<'a>(
    cpu: &'a CpuInfo,
    cpu_history: &'a [f32],
    unit: TemperatureUnit,
    is_dark: bool,
    expanded_cores: bool,
) -> Element<'a, Message> {
    let sp = cosmic::theme::spacing();
    let status = ThermalStatus::from_celsius(cpu.package_temp);
    let [r, g, b] = status.rgb();
    let state_color = Color::from_rgb(r, g, b);

    // 1. Navigation Breadcrumb Header
    let back_btn = button::text("← Overview")
        .padding([6, 12])
        .class(cosmic::theme::Button::Standard)
        .on_press(Message::SelectTab(ActiveTab::Overview));

    let cpu_title = text::title3(&cpu.model).size(14);
    let nav_row = row![back_btn, cosmic::iced::widget::Space::new().width(Length::Fill), cpu_title]
        .spacing(sp.space_s)
        .align_y(Alignment::Center)
        .width(Length::Fill);

    // 2. 3 Big Metrics: Temperature (BIG), Load (BIG), Speed (BIG)
    let big_stat_card = |label: &'static str, value: String, accent: Option<Color>| {
        let val_text = text::title1(value).size(26);
        let lbl_text = text::caption(label).size(11);

        let content = column![lbl_text, val_text]
            .spacing(2)
            .align_x(Alignment::Center);

        container(content)
            .padding([12, 14])
            .width(Length::FillPortion(1))
            .align_x(cosmic::iced::alignment::Horizontal::Center)
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

    let temp_str = unit.format_temp(cpu.package_temp);
    let load_str = format!("{:.1}%", cpu.overall_usage);
    let speed_ghz = cpu.avg_freq_mhz as f32 / 1000.0;
    let speed_str = format!("{:.2} GHz", speed_ghz);

    let big_metrics_row = row![
        big_stat_card("Temperature", temp_str, Some(state_color)),
        big_stat_card("Load / Utilization", load_str, None),
        big_stat_card("Clock Speed", speed_str, None),
    ]
    .spacing(sp.space_s)
    .width(Length::Fill);

    // 3. Telemetry Info Strip: Power Draw, Processes, Threads, Handles
    let mini_info = |label: &'static str, val: String| {
        row![
            text::caption(label).size(11),
            text::body(val).size(11),
        ]
        .spacing(4)
        .align_y(Alignment::Center)
    };

    let power_val = match cpu.power_watts {
        Some(w) => format!("{:.1} W", w),
        None => "Estimating...".to_string(),
    };

    let telemetry_strip = row![
        mini_info("Power Draw:", power_val),
        text::caption("·").size(11),
        mini_info("Processes:", cpu.num_processes.to_string()),
        text::caption("·").size(11),
        mini_info("Threads:", cpu.num_threads.to_string()),
        text::caption("·").size(11),
        mini_info("Handles:", cpu.num_handles.to_string()),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let telemetry_container = container(telemetry_strip)
        .padding([6, 12])
        .align_x(cosmic::iced::alignment::Horizontal::Center)
        .width(Length::Fill)
        .class(cosmic::theme::Container::Custom(Box::new(move |_theme| {
            cosmic::iced::widget::container::Style {
                background: Some(if is_dark {
                    Color::from_rgba(1.0, 1.0, 1.0, 0.02).into()
                } else {
                    Color::from_rgba(0.0, 0.0, 0.0, 0.02).into()
                }),
                border: cosmic::iced::Border {
                    color: if is_dark {
                        Color::from_rgba(1.0, 1.0, 1.0, 0.05)
                    } else {
                        Color::from_rgba(0.0, 0.0, 0.0, 0.06)
                    },
                    width: 1.0,
                    radius: 8.0.into(),
                },
                ..Default::default()
            }
        })));

    // 4. Curve: CPU Package Temperature & Activity Curve
    let sparkline = Sparkline::new(cpu_history, state_color).view(Length::Fill, Length::Fixed(72.0));
    let graph_label = text::caption("CPU Temperature Curve").size(11);
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

    // 5. Cores Grid: Equal-width cards in 2 balanced rows
    let total_cores = cpu.cores.len();
    let mid = (total_cores + 1) / 2;

    let render_card = |core: &'a crate::hardware::types::CpuCoreInfo| {
        let core_status = ThermalStatus::from_celsius(core.temp);
        let [cr, cg, cb] = core_status.rgb();
        let fill_color = Color::from_rgb(cr, cg, cb);

        let label = text::caption(&core.label).size(10);

        let fraction = ((core.temp - 25.0) / 60.0).clamp(0.08, 1.0);
        let bar = VerticalBar::new(fraction, fill_color).view(10.0, 54.0);

        let temp_str = unit.format_temp_short(core.temp);
        let temp_txt = text::title3(temp_str).size(13);

        let usage_str = format!("{:.0}%", core.usage_percent);
        let usage_txt = text::caption(usage_str).size(10);

        let mut card_content = column![label, bar, temp_txt, usage_txt]
            .spacing(3)
            .align_x(Alignment::Center);

        if let Some(freq) = core.freq_mhz {
            let freq_txt = text::caption(format!("{:.1}G", freq as f32 / 1000.0)).size(9);
            card_content = card_content.push(freq_txt);
        }

        container(card_content)
            .padding([6, 4])
            .width(Length::FillPortion(1))
            .align_x(cosmic::iced::alignment::Horizontal::Center)
            .class(cosmic::theme::Container::Custom(Box::new(move |_theme| {
                cosmic::iced::widget::container::Style {
                    background: Some(if is_dark {
                        Color::from_rgba(1.0, 1.0, 1.0, 0.035).into()
                    } else {
                        Color::from_rgba(0.0, 0.0, 0.0, 0.035).into()
                    }),
                    border: cosmic::iced::Border {
                        color: if is_dark {
                            Color::from_rgba(1.0, 1.0, 1.0, 0.07)
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

    let mut row1 = row![].spacing(8).width(Length::Fill).align_y(Alignment::Center);
    for core in cpu.cores.iter().take(mid) {
        row1 = row1.push(render_card(core));
    }

    let mut row2 = row![].spacing(8).width(Length::Fill).align_y(Alignment::Center);
    for core in cpu.cores.iter().skip(mid) {
        row2 = row2.push(render_card(core));
    }

    let cores_grid = column![row1, row2]
        .spacing(8)
        .align_x(Alignment::Center)
        .width(Length::Fill);

    // 6. Cores Collapsible Dropdown Header
    let arrow_icon = if expanded_cores { "⌃" } else { "⌄" };
    let cores_dropdown = mouse_area(
        container(
            row![
                text(format!("All {} Cores", total_cores)).size(12),
                cosmic::iced::widget::Space::new().width(Length::Fill),
                text(arrow_icon).size(15),
            ]
            .align_y(Alignment::Center)
        )
        .padding([8, 14])
        .width(Length::Fill)
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
                    radius: 10.0.into(),
                },
                ..Default::default()
            }
        })))
    )
    .on_press(Message::ToggleCpuCores)
    .interaction(cosmic::iced::mouse::Interaction::Pointer);

    let mut main_col = column![nav_row, big_metrics_row, telemetry_container, graph_card, cores_dropdown]
        .spacing(sp.space_s)
        .width(Length::Fill);

    if expanded_cores {
        main_col = main_col.push(cores_grid);
    }

    main_col.into()
}
