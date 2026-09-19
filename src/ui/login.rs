use crate::app::Message;
use crate::ui::icons::Icon;
use crate::ui::theme;
use iced::{
    Alignment, Background, Border, Color, Element, Length, Theme,
    widget::{Button, Column, Container, Image, Row, Space, Text},
};

const LOGO_BYTES: &[u8] = include_bytes!("../../assets/spotifust.png");

#[allow(clippy::too_many_lines, clippy::cast_precision_loss)]
pub fn view(is_loading: bool, error: Option<&str>, animation_tick: u32) -> Element<'_, Message> {
    let logo_handle = iced::widget::image::Handle::from_bytes(LOGO_BYTES);
    let logo = Image::new(logo_handle)
        .width(Length::Fixed(84.0))
        .height(Length::Fixed(84.0))
        .filter_method(iced::widget::image::FilterMethod::Linear);

    let title = Text::new("Spotifust")
        .size(36)
        .font(iced::Font {
            weight: iced::font::Weight::Bold,
            ..Default::default()
        })
        .color(theme::TEXT_PRIMARY);

    let badge = Container::new(
        Text::new("HIGH FIDELITY")
            .size(10)
            .font(iced::Font {
                weight: iced::font::Weight::Bold,
                ..Default::default()
            })
            .color(theme::SPOTIFY_GREEN),
    )
    .padding([3, 8])
    .style(|_theme: &Theme| iced::widget::container::Style {
        background: Some(Background::Color(Color {
            r: theme::SPOTIFY_GREEN.r,
            g: theme::SPOTIFY_GREEN.g,
            b: theme::SPOTIFY_GREEN.b,
            a: 0.15,
        })),
        border: Border {
            color: Color {
                r: theme::SPOTIFY_GREEN.r,
                g: theme::SPOTIFY_GREEN.g,
                b: theme::SPOTIFY_GREEN.b,
                a: 0.4,
            },
            width: 1.0,
            radius: theme::RADIUS_PILL.into(),
        },
        ..Default::default()
    });

    let header_row = Row::new()
        .align_y(Alignment::Center)
        .spacing(12)
        .push(title)
        .push(badge);

    let subtitle = Text::new("Ultra-fast native Spotify client • Pure Rust • No web bloat")
        .size(13)
        .color(theme::TEXT_SECONDARY);

    let mut eq_row = Row::new().spacing(4).align_y(Alignment::End);
    for i in 0..7 {
        let tick_phase = (animation_tick % 1000) as f32 * 0.18;
        let bar_offset = i as f32 * 0.95;
        let factor = f32::midpoint((tick_phase + bar_offset).sin(), 1.0);
        let bar_height = 5.0 + factor * 22.0;
        let bar_color = if i % 2 == 0 {
            theme::SPOTIFY_GREEN
        } else {
            theme::ACCENT
        };
        let bar = Container::new(Space::new())
            .width(Length::Fixed(4.0))
            .height(Length::Fixed(bar_height))
            .style(move |_theme: &Theme| iced::widget::container::Style {
                background: Some(Background::Color(bar_color)),
                border: Border {
                    radius: theme::RADIUS_SM.into(),
                    ..Default::default()
                },
                ..Default::default()
            });
        eq_row = eq_row.push(bar);
    }

    let mut inner_col = Column::new()
        .spacing(14)
        .align_x(Alignment::Center)
        .push(logo)
        .push(header_row)
        .push(subtitle)
        .push(Container::new(eq_row).padding([6, 0]));

    if let Some(err) = error {
        let is_session_expired = err.to_lowercase().contains("expired")
            || err.to_lowercase().contains("expiró")
            || err.to_lowercase().contains("session");

        let (alert_bg, alert_border, alert_color) = if is_session_expired {
            (
                Color {
                    r: 0.961,
                    g: 0.620,
                    b: 0.043,
                    a: 0.12,
                },
                Color {
                    r: 0.961,
                    g: 0.620,
                    b: 0.043,
                    a: 0.45,
                },
                Color {
                    r: 0.984,
                    g: 0.749,
                    b: 0.141,
                    a: 1.0,
                },
            )
        } else {
            (
                Color {
                    r: theme::COLOR_ERROR.r,
                    g: theme::COLOR_ERROR.g,
                    b: theme::COLOR_ERROR.b,
                    a: 0.12,
                },
                Color {
                    r: theme::COLOR_ERROR.r,
                    g: theme::COLOR_ERROR.g,
                    b: theme::COLOR_ERROR.b,
                    a: 0.45,
                },
                Color {
                    r: 0.973,
                    g: 0.443,
                    b: 0.443,
                    a: 1.0,
                },
            )
        };

        let alert_icon = if is_session_expired {
            Icon::Clock
        } else {
            Icon::X
        };

        inner_col = inner_col.push(
            Container::new(
                Row::new()
                    .align_y(Alignment::Center)
                    .spacing(10)
                    .push(alert_icon.view_colored(16.0, alert_color))
                    .push(Text::new(err).color(alert_color).size(13).font(iced::Font {
                        weight: iced::font::Weight::Medium,
                        ..Default::default()
                    })),
            )
            .padding([10, 16])
            .width(Length::Fill)
            .style(move |_theme: &Theme| iced::widget::container::Style {
                background: Some(Background::Color(alert_bg)),
                border: Border {
                    color: alert_border,
                    width: 1.0,
                    radius: theme::RADIUS_MD.into(),
                },
                ..Default::default()
            }),
        );
    }

    if is_loading {
        let dots_count = (animation_tick / 3) % 4;
        let dots = match dots_count {
            0 => "",
            1 => ".",
            2 => "..",
            _ => "...",
        };
        let loading_text = format!("Awaiting browser login{dots}");

        let loading_card = Column::new()
            .spacing(12)
            .align_x(Alignment::Center)
            .push(
                Row::new()
                    .align_y(Alignment::Center)
                    .spacing(10)
                    .push(Icon::Devices.view_colored(20.0, theme::SPOTIFY_GREEN))
                    .push(
                        Text::new(loading_text)
                            .size(15)
                            .font(iced::Font {
                                weight: iced::font::Weight::Bold,
                                ..Default::default()
                            })
                            .color(theme::TEXT_PRIMARY),
                    ),
            )
            .push(
                Text::new("Authorize Spotifust in your opened default web browser.")
                    .size(12)
                    .color(theme::TEXT_SECONDARY),
            )
            .push(
                Button::new(
                    Text::new("Cancel")
                        .size(12)
                        .font(iced::Font {
                            weight: iced::font::Weight::Medium,
                            ..Default::default()
                        })
                        .color(theme::TEXT_SECONDARY),
                )
                .on_press(Message::CancelLogin)
                .padding([6, 18])
                .style(|_theme: &Theme, status| {
                    let base = iced::widget::button::Style {
                        background: Some(Background::Color(theme::SURFACE_HOVER)),
                        border: Border {
                            radius: theme::RADIUS_PILL.into(),
                            color: theme::BORDER_SUBTLE,
                            width: 1.0,
                        },
                        ..Default::default()
                    };
                    match status {
                        iced::widget::button::Status::Hovered => iced::widget::button::Style {
                            background: Some(Background::Color(theme::SURFACE_ACTIVE)),
                            ..base
                        },
                        _ => base,
                    }
                }),
            );

        inner_col = inner_col.push(
            Container::new(loading_card)
                .padding([18, 24])
                .width(Length::Fill)
                .style(|_theme: &Theme| iced::widget::container::Style {
                    background: Some(Background::Color(theme::SURFACE_HOVER)),
                    border: Border {
                        color: theme::BORDER_SUBTLE,
                        width: 1.0,
                        radius: theme::RADIUS_LG.into(),
                    },
                    ..Default::default()
                }),
        );
    } else {
        let login_btn = Button::new(
            Row::new()
                .align_y(Alignment::Center)
                .spacing(10)
                .push(Icon::Play.view_colored(16.0, Color::BLACK))
                .push(
                    Text::new("LOG IN WITH SPOTIFY")
                        .size(14)
                        .font(iced::Font {
                            weight: iced::font::Weight::Bold,
                            ..Default::default()
                        })
                        .color(Color::BLACK),
                ),
        )
        .on_press(Message::LoginRequested)
        .padding([15, 38])
        .style(|_theme: &Theme, status| {
            let base = iced::widget::button::Style {
                background: Some(Background::Color(theme::SPOTIFY_GREEN)),
                text_color: Color::BLACK,
                border: Border {
                    radius: theme::RADIUS_PILL.into(),
                    ..Default::default()
                },
                ..Default::default()
            };

            match status {
                iced::widget::button::Status::Hovered => iced::widget::button::Style {
                    background: Some(Background::Color(theme::SPOTIFY_GREEN_HOVER)),
                    ..base
                },
                iced::widget::button::Status::Pressed => iced::widget::button::Style {
                    background: Some(Background::Color(theme::SPOTIFY_GREEN_PRESSED)),
                    ..base
                },
                _ => base,
            }
        });

        inner_col = inner_col.push(Container::new(login_btn).padding([8, 0]));
    }

    let feature_badge = |icon: Icon, title: &'static str, subtitle: &'static str| {
        Container::new(
            Column::new()
                .spacing(2)
                .align_x(Alignment::Center)
                .push(icon.view_colored(18.0, theme::SPOTIFY_GREEN))
                .push(
                    Text::new(title)
                        .size(11)
                        .font(iced::Font {
                            weight: iced::font::Weight::Bold,
                            ..Default::default()
                        })
                        .color(theme::TEXT_PRIMARY),
                )
                .push(Text::new(subtitle).size(10).color(theme::TEXT_TERTIARY)),
        )
        .padding([8, 12])
        .width(Length::FillPortion(1))
        .style(|_theme: &Theme| iced::widget::container::Style {
            background: Some(Background::Color(Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.03,
            })),
            border: Border {
                radius: theme::RADIUS_MD.into(),
                color: theme::BORDER_SUBTLE,
                width: 1.0,
            },
            ..Default::default()
        })
    };

    let features_row = Row::new()
        .spacing(8)
        .width(Length::Fill)
        .push(feature_badge(Icon::MusicNote, "320 kbps", "Bitrate Audio"))
        .push(feature_badge(Icon::Devices, "< 25 MB", "Memory Usage"))
        .push(feature_badge(Icon::Lock, "PKCE Flow", "Keyring Vault"));

    inner_col = inner_col.push(Container::new(features_row).padding([6, 0]));

    let footer = Text::new("Powered by Librespot & RSpotify • Zero browser engine overhead")
        .size(11)
        .color(theme::TEXT_TERTIARY);

    inner_col = inner_col.push(footer);

    let card = Container::new(inner_col)
        .padding(40)
        .max_width(500.0)
        .style(|_theme: &Theme| iced::widget::container::Style {
            background: Some(Background::Color(theme::SURFACE_CARD)),
            border: Border {
                radius: theme::RADIUS_XL.into(),
                color: Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 0.08,
                },
                width: 1.0,
            },
            ..Default::default()
        });

    Container::new(card)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(|_theme: &Theme| iced::widget::container::Style {
            background: Some(Background::Color(theme::BG_BASE)),
            ..Default::default()
        })
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_login_view_idle_renders_without_panic() {
        let _ = view(false, None, 0);
    }

    #[test]
    fn test_login_view_loading_renders_without_panic() {
        let _ = view(true, None, 5);
    }

    #[test]
    fn test_login_view_error_renders_without_panic() {
        let _ = view(false, Some("Session expired. Please log in again."), 10);
    }
}
