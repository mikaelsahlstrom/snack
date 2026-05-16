use crate::app::{ AppState, Snack };
use crate::message::Message;
use crate::xmpp;

impl Snack
{
    pub(crate) fn subscription(&self) -> iced::Subscription<Message>
    {
        let keyboard = iced::keyboard::listen().map(|event|
        {
            if let iced::keyboard::Event::KeyPressed { key, modifiers, .. } = event
            {
                if key == iced::keyboard::Key::Named(iced::keyboard::key::Named::Tab)
                {
                    if modifiers.shift()
                    {
                        return Message::ShiftTabPressed;
                    }
                    return Message::TabPressed;
                }

                if modifiers.alt() && !modifiers.shift() && !modifiers.control() && !modifiers.command()
                {
                    if key == iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowUp)
                    {
                        return Message::PrevSelection;
                    }
                    if key == iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowDown)
                    {
                        return Message::NextSelection;
                    }
                }

                let primary = (modifiers.command() || modifiers.control()) && !modifiers.shift() && !modifiers.alt();
                if primary
                {
                    if key == iced::keyboard::Key::Character("n".into())
                    {
                        return Message::ShowJoinPanel;
                    }
                    if key == iced::keyboard::Key::Character("w".into())
                    {
                        return Message::LeaveSelection;
                    }
                }
            }

            return Message::Ignore;
        });

        let window_focus = iced::event::listen_with(|event, _status, _id|
        {
            match event
            {
                iced::Event::Window(iced::window::Event::Focused) => Some(Message::WindowFocused),
                iced::Event::Window(iced::window::Event::Unfocused) => Some(Message::WindowUnfocused),
                _ => None,
            }
        });

        match (&self.state, &self.xmpp_cmd_rx)
        {
            (AppState::Connecting | AppState::Connected, Some(cmd_rx)) =>
            {
                let xmpp_sub = iced::Subscription::run_with(
                    cmd_rx.clone(),
                    |cmd: &xmpp::CommandChannel|
                    {
                        xmpp::connect(cmd.clone())
                    },
                ).map(Message::XmppEvent);

                return iced::Subscription::batch([keyboard, window_focus, xmpp_sub]);
            }
            _ => return iced::Subscription::batch([keyboard, window_focus]),
        }
    }
}
