use iced::{ Element, Fill, Length };
use iced::widget::{ button, column, container, text, text_input, Id };

use crate::{ AppState, Message, Snack, ACCOUNT_JID_INPUT_ID, ACCOUNT_PASSWORD_INPUT_ID };

pub fn view(state: &Snack) -> Element<'_, Message>
{
    let connecting = state.state == AppState::Connecting;

    let mut jid_input = text_input("user@example.org", &state.jid_input)
        .id(Id::new(ACCOUNT_JID_INPUT_ID))
        .padding(10)
        .width(Length::Fixed(400.0));

    let mut password_input = text_input("Password", &state.password_input)
        .id(Id::new(ACCOUNT_PASSWORD_INPUT_ID))
        .secure(true)
        .padding(10)
        .width(Length::Fixed(400.0));

    if !connecting
    {
        jid_input = jid_input
            .on_input(Message::JidInputChanged)
            .on_submit(Message::FocusPassword);
        password_input = password_input
            .on_input(Message::PasswordInputChanged)
            .on_submit(Message::Connect);
    }

    let heading = if connecting
    {
        text("Connecting...").size(18)
    }
    else
    {
        text("Connect to account").size(18)
    };

    let mut connect_btn = button(text("Connect").size(14)).padding(10);
    if !connecting
    {
        connect_btn = connect_btn.on_press(Message::Connect);
    }

    let mut form = column![
        heading,
        jid_input,
        password_input,
        connect_btn,
    ].spacing(12).align_x(iced::Alignment::Center);

    if let Some(ref err) = state.connect_error
    {
        form = form.push(text(err.clone()).size(14));
    }

    return container(form)
            .center(Fill)
            .width(Fill)
            .height(Fill)
            .into();
}
