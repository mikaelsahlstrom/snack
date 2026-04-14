use iced::{ Element, Fill, Length };
use iced::widget::{ column, container, scrollable, text };

use crate::{ Message, Snack };

pub fn view(state: &Snack) -> Element<'_, Message>
{
    if let Some(index) = state.active_room
    {
        let room = &state.rooms[index];
        let member_count = room.users.len();
        let members: Vec<Element<'_, Message>> = room.users.iter().map(|u|
        {
            text(&u.name).size(14).into()
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
