use iced::{ Element, Fill, Length };
use iced::widget::{ button, column, container, row, text, text_input, Id };

use crate::{ Message, Snack, JOIN_INPUT_ID };

pub fn view(state: &Snack) -> Element<'_, Message>
{
    let joining = state.joining_room.is_some();

    let heading = if joining
    {
        text("Joining...").size(18)
    }
    else
    {
        text("Join a room").size(18)
    };

    let mut join_input = text_input("room@conference.example.org", &state.join_input)
        .id(Id::new(JOIN_INPUT_ID))
        .padding(10)
        .width(Length::Fixed(400.0));

    if !joining
    {
        join_input = join_input
            .on_input(Message::JoinInputChanged)
            .on_submit(Message::JoinRoom);
    }

    let mut join_btn = button(text("Join").size(14)).padding(10);
    if !joining
    {
        join_btn = join_btn.on_press(Message::JoinRoom);
    }

    let mut buttons = row![].spacing(8);
    if state.active_room.is_some() && !joining
    {
        let cancel_btn = button(text("Cancel").size(14))
            .on_press(Message::HideJoinPanel)
            .padding(10)
            .style(button::text);
        buttons = buttons.push(cancel_btn);
    }
    buttons = buttons.push(join_btn);

    let mut form = column![
        heading,
        join_input,
        buttons,
    ].spacing(12).align_x(iced::Alignment::Center);

    if let Some(ref err) = state.join_error
    {
        form = form.push(text(err.clone()).size(14));
    }

    return container(form)
            .center(Fill)
            .width(Fill)
            .height(Fill)
            .into();
}
