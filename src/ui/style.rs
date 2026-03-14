use iced::Theme;
use iced::widget::button;

pub fn room_button_active(theme: &Theme, status: button::Status) -> button::Style
{
    let palette = theme.extended_palette();
    let base = button::text(theme, status);

    button::Style
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
    }
}
