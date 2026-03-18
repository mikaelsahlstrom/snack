use iced::{ Element, Fill, Length };
use iced::widget::{ button, column, container, text, text_input, Id };

use crate::{ Message, Snack, ACCOUNT_JID_INPUT_ID, ACCOUNT_PASSWORD_INPUT_ID };

pub fn view(state: &Snack) -> Element<'_, Message>
{
    let jid_input = text_input("user@example.org", &state.jid_input)
        .id(Id::new(ACCOUNT_JID_INPUT_ID))
        .on_input(Message::JidInputChanged)
        .on_submit(Message::FocusPassword)
        .padding(10)
        .width(Length::Fixed(400.0));

    let password_input = text_input("Password", &state.password_input)
        .id(Id::new(ACCOUNT_PASSWORD_INPUT_ID))
        .on_input(Message::PasswordInputChanged)
        .on_submit(Message::Connect)
        .secure(true)
        .padding(10)
        .width(Length::Fixed(400.0));

    let connect_btn = button(text("Connect").size(14))
        .on_press(Message::Connect)
        .padding(10);

    let mut form = column![
        text("Connect to account").size(18),
        jid_input,
        password_input,
        connect_btn,
    ].spacing(12).align_x(iced::Alignment::Center);

    if let Some(ref err) = state.connect_error
    {
        form = form.push(text(err.clone()).size(14));
    }

    container(form)
        .center(Fill)
        .width(Fill)
        .height(Fill)
        .into()
}
