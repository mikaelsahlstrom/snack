use iced::{ Element, Fill, Length };
use iced::widget::{ column, container, scrollable, text, tooltip };

use crate::{ Message, Snack };

pub fn view(state: &Snack) -> Element<'_, Message>
{
    if let Some(index) = state.active_room
    {
        let room = &state.rooms[index];
        let member_count = room.users.len();
        let members: Vec<Element<'_, Message>> = room.users.iter().map(|u|
        {
            let show_indicator = match u.show.as_deref()
            {
                Some("away") | Some("xa") => " (away)",
                Some("dnd") => " (busy)",
                _ => "",
            };

            let label = if show_indicator.is_empty()
            {
                u.name.clone()
            }
            else
            {
                format!("{}{}", u.name, show_indicator)
            };

            if let Some(ref jid) = u.jid
            {
                tooltip(
                    text(label.clone()).size(14),
                    container(text(jid).size(12)).padding(4).style(container::bordered_box),
                    tooltip::Position::Right,
                ).into()
            }
            else
            {
                text(label).size(14).into()
            }
        }).collect();

        return container(
            column![
                text(format!("Members ({})", member_count)).size(12),
                scrollable(
                    column(members).spacing(6).width(Fill)
                )
            ].spacing(8).width(Fill)
        )
        .width(Length::Fixed(160.0))
        .height(Fill)
        .padding(8)
        .style(container::bordered_box)
        .into();
    }
    else
    {
        return container(text(""))
                .width(Length::Fixed(160.0))
                .height(Fill)
                .into();
    }
}
