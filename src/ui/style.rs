use iced::{ Color, Theme };
use iced::widget::{ button, container };

pub fn mention_highlight(_theme: &Theme) -> container::Style
{
    return container::Style
    {
        background: Some(iced::Background::Color(
            Color::from_rgba(1.0, 0.85, 0.35, 0.12)
        )),
        ..Default::default()
    };
}

pub fn room_button_active(theme: &Theme, status: button::Status) -> button::Style
{
    let palette = theme.extended_palette();
    let base = button::text(theme, status);

    return button::Style
    {
        background: Some(iced::Background::Color(
            match status
            {
                button::Status::Hovered => palette.primary.weak.color,
                _ => palette.primary.weak.color.scale_alpha(0.5),
            }
        )),
        text_color: palette.primary.strong.text,
        ..base
    };
}
