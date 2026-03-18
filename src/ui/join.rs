use iced::{ Element, Fill, Length };
use iced::widget::{ button, column, container, row, text, text_input, Id };

use crate::{ Message, Snack, JOIN_INPUT_ID };

pub fn view(state: &Snack) -> Element<'_, Message>
{
    let join_input = text_input("room@conference.example.org", &state.join_input)
        .id(Id::new(JOIN_INPUT_ID))
        .on_input(Message::JoinInputChanged)
        .on_submit(Message::JoinRoom)
        .padding(10)
        .width(Length::Fixed(400.0));

    let join_btn = button(text("Join").size(14))
        .on_press(Message::JoinRoom)
        .padding(10);

    let mut buttons = row![].spacing(8);
    if state.active_room.is_some()
    {
        let cancel_btn = button(text("Cancel").size(14))
            .on_press(Message::HideJoinPanel)
            .padding(10)
            .style(button::text);
        buttons = buttons.push(cancel_btn);
    }
    buttons = buttons.push(join_btn);

    container(
        column![
            text("Join a room").size(18),
            join_input,
            buttons,
        ].spacing(12).align_x(iced::Alignment::Center)
    )
    .center(Fill)
    .width(Fill)
    .height(Fill)
    .into()
}
