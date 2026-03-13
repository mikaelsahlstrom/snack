use iced::{ Application, Element, Program, Task, Theme, Length, Fill };
use iced::widget::{ column, container, row, scrollable, text, text_input, button, Id };

mod room;

const MESSAGE_SCROLL_ID: &str = "message_scroll";
const MESSAGE_INPUT_ID: &str = "message_input";
const JOIN_INPUT_ID: &str = "join_input";

fn focus_join_input() -> Task<Message>
{
    iced::widget::operation::focus(Id::new(JOIN_INPUT_ID))
}

fn focus_input() -> Task<Message>
{
    iced::widget::operation::focus(Id::new(MESSAGE_INPUT_ID))
}

fn snap_to_bottom() -> Task<Message>
{
    iced::widget::operation::snap_to_end(Id::new(MESSAGE_SCROLL_ID))
}

fn room_button_active(theme: &Theme, status: button::Status) -> button::Style
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

struct Snack
{
    rooms: Vec<room::Room>,
    active_room: Option<usize>,
    message_input: String,
    show_join_panel: bool,
    join_input: String,
}

#[derive(Debug, Clone)]
enum Message
{
    SelectRoom(usize),
    InputChanged(String),
    SendMessage,
    ShowJoinPanel,
    HideJoinPanel,
    JoinInputChanged(String),
    JoinRoom,
    LeaveRoom,
}

fn main() -> iced::Result
{
    return application().run();
}

fn application() -> Application<impl Program<Message = Message, Theme = Theme>>
{
    return iced::application(Snack::new, Snack::update, Snack::view)
                .subscription(Snack::subscription)
                .title(Snack::title)
                .theme(Snack::theme);
}

impl Snack
{
    fn new() -> (Self, Task<Message>)
    {
        let rooms = vec![
            room::Room
            {
                jid: "rust@chat.example.org".to_string(),
                title: "Rust".to_string(),
                topic: "Rust programming discussion".to_string(),
                users: vec![
                    room::user::User { jid: "alice@example.org".into(), name: "Alice".into() },
                    room::user::User { jid: "bob@example.org".into(), name: "Bob".into() },
                ],
                messages: vec![
                    room::message::Message { from: "Alice".into(), body: "Hello everyone!".into(), received: chrono::Utc::now() },
                    room::message::Message { from: "Bob".into(), body: "Hey Alice!".into(), received: chrono::Utc::now() },
                ],
                unread: true,
            },
            room::Room
            {
                jid: "xmpp@chat.example.org".to_string(),
                title: "XMPP".to_string(),
                topic: "XMPP discussion".to_string(),
                users: vec![
                    room::user::User { jid: "charlie@example.org".into(), name: "Charlie".into() },
                ],
                messages: vec![
                    room::message::Message { from: "Charlie".into(), body: "Welcome to XMPP!".into(), received: chrono::Utc::now() },
                ],
                unread: false,
            },
            room::Room
            {
                jid: "linux@conference.jabber.org".to_string(),
                title: "Linux".to_string(),
                topic: "Linux and open source".to_string(),
                users: vec![
                    room::user::User { jid: "dave@jabber.org".into(), name: "Dave".into() },
                    room::user::User { jid: "eve@jabber.org".into(), name: "Eve".into() },
                ],
                messages: vec![
                    room::message::Message { from: "Dave".into(), body: "Anyone tried the new kernel?".into(), received: chrono::Utc::now() },
                    room::message::Message { from: "Eve".into(), body: "Yes, it's great!".into(), received: chrono::Utc::now() },
                ],
                unread: true,
            },
            room::Room
            {
                jid: "gaming@conference.jabber.org".to_string(),
                title: "Gaming".to_string(),
                topic: "PC and console gaming".to_string(),
                users: vec![
                    room::user::User { jid: "frank@jabber.org".into(), name: "Frank".into() },
                ],
                messages: vec![
                    room::message::Message { from: "Frank".into(), body: "What are you all playing?".into(), received: chrono::Utc::now() },
                ],
                unread: false,
            },
        ];

        (Self
        {
            rooms,
            active_room: Some(0),
            message_input: String::new(),
            show_join_panel: false,
            join_input: String::new(),
        }, focus_input())
    }

    fn title(&self) -> String
    {
        return "Snack".to_string();
    }

    fn update(&mut self, message: Message) -> Task<Message>
    {
        match message
        {
            Message::SelectRoom(index) =>
            {
                self.active_room = Some(index);
                self.show_join_panel = false;
                return Task::batch([snap_to_bottom(), focus_input()]);
            }
            Message::InputChanged(value) =>
            {
                self.message_input = value;
            }
            Message::SendMessage =>
            {
                if let Some(index) = self.active_room
                {
                    let body = self.message_input.trim().to_string();
                    if !body.is_empty()
                    {
                        self.rooms[index].messages.push(room::message::Message
                        {
                            from: "You".to_string(),
                            body,
                            received: chrono::Utc::now(),
                        });

                        self.message_input.clear();

                        return Task::batch([snap_to_bottom(), focus_input()]);
                    }
                }
            }
            Message::ShowJoinPanel =>
            {
                self.show_join_panel = true;
                self.join_input.clear();
                return focus_join_input();
            }
            Message::HideJoinPanel =>
            {
                self.show_join_panel = false;
                return focus_input();
            }
            Message::JoinInputChanged(value) =>
            {
                self.join_input = value;
            }
            Message::JoinRoom =>
            {
                let jid = self.join_input.trim().to_string();
                if !jid.is_empty()
                {
                    let title = jid.split('@').next().unwrap_or(&jid).to_string();
                    self.rooms.push(room::Room
                    {
                        jid,
                        title,
                        topic: String::new(),
                        users: Vec::new(),
                        messages: Vec::new(),
                        unread: false,
                    });
                    self.active_room = Some(self.rooms.len() - 1);
                    self.show_join_panel = false;
                    self.join_input.clear();
                    return focus_input();
                }
            }
            Message::LeaveRoom =>
            {
                if let Some(index) = self.active_room
                {
                    self.rooms.remove(index);
                    if self.rooms.is_empty()
                    {
                        self.active_room = None;
                    }
                    else if index >= self.rooms.len()
                    {
                        self.active_room = Some(self.rooms.len() - 1);
                    }
                    return focus_input();
                }
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message>
    {
        // Room list (left sidebar).
        let room_list: Element<'_, Message> =
        {
            let sidebar_header = row![
                text("Rooms").size(14),
                text("").width(Fill),
                button(text("+").size(14))
                    .on_press(Message::ShowJoinPanel)
                    .padding(4)
                    .style(button::text),
            ].align_y(iced::Alignment::Center).width(Fill);

            // Group rooms by server (part after @ in jid).
            let mut servers: Vec<String> = Vec::new();
            let mut grouped: Vec<(String, Vec<usize>)> = Vec::new();

            for (i, r) in self.rooms.iter().enumerate()
            {
                let server = r.jid.split('@').nth(1).unwrap_or(&r.jid).to_string();
                if let Some(pos) = servers.iter().position(|s| *s == server)
                {
                    grouped[pos].1.push(i);
                }
                else
                {
                    servers.push(server.clone());
                    grouped.push((server, vec![i]));
                }
            }

            let mut items: Vec<Element<'_, Message>> = Vec::new();
            for (server, indices) in &grouped
            {
                // Server header.
                items.push(
                    text(server.clone()).size(12).into()
                );

                // Room entries with unread dot or empty space, aligned with server names.
                for &i in indices
                {
                    let r = &self.rooms[i];
                    let is_active = self.active_room == Some(i);
                    let icon = if r.unread { "\u{2022}" } else { " " };
                    let label = row![
                        text(icon).size(14),
                        text(&r.title).size(14),
                    ].spacing(6).align_y(iced::Alignment::Center);

                    let style = if is_active { room_button_active } else { button::text };

                    let item = button(label)
                        .on_press(Message::SelectRoom(i))
                        .width(Fill)
                        .padding(6)
                        .style(style);

                    items.push(item.into());
                }
            }

            let list = scrollable(
                column(items).spacing(2).width(Fill)
            );

            container(
                column![sidebar_header, list].spacing(8).width(Fill)
            )
            .width(Length::Fixed(200.0))
            .height(Fill)
            .padding(8)
            .style(container::bordered_box)
            .into()
        };

        // Center: topic, messages and input.
        let center: Element<'_, Message> = if self.show_join_panel
        {
            let join_input = text_input("room@conference.example.org", &self.join_input)
                .id(Id::new(JOIN_INPUT_ID))
                .on_input(Message::JoinInputChanged)
                .on_submit(Message::JoinRoom)
                .padding(10)
                .width(Length::Fixed(400.0));

            let join_btn = button(text("Join").size(14))
                .on_press(Message::JoinRoom)
                .padding(10);

            let cancel_btn = button(text("Cancel").size(14))
                .on_press(Message::HideJoinPanel)
                .padding(10)
                .style(button::text);

            container(
                column![
                    text("Join a room").size(18),
                    join_input,
                    row![cancel_btn, join_btn].spacing(8),
                ].spacing(12).align_x(iced::Alignment::Center)
            )
            .center(Fill)
            .width(Fill)
            .height(Fill)
            .into()
        }
        else if let Some(index) = self.active_room
        {
            let room = &self.rooms[index];

            // Room topic with leave button.
            let leave_btn = button(text("Leave").size(12))
                .on_press(Message::LeaveRoom)
                .padding(4)
                .style(button::text);

            let topic_label = container(
                row![
                    text(&room.topic).size(14),
                    text("").width(Fill),
                    leave_btn,
                ].align_y(iced::Alignment::Center).width(Fill)
            )
                .padding(8)
                .width(Fill)
                .style(container::bordered_box);

            // Message list.
            let today = chrono::Local::now().date_naive();
            let messages: Vec<Element<'_, Message>> = room.messages.iter().map(|m|
            {
                let local_time = m.received.with_timezone(&chrono::Local);
                let timestamp = if local_time.date_naive() == today
                {
                    local_time.format("%H:%M:%S").to_string()
                }
                else
                {
                    local_time.format("%Y-%m-%d %H:%M:%S").to_string()
                };
                let line = text(format!("[{}] {}: {}", timestamp, m.from, m.body)).size(14);

                container(line).padding(4).width(Fill).into()
            }).collect();

            let message_area = scrollable(
                column(messages).spacing(2).width(Fill)
            )
            .id(Id::new(MESSAGE_SCROLL_ID))
            .height(Fill)
            .width(Fill);

            // Input bar.
            let input = text_input("Type a message...", &self.message_input)
                .id(Id::new(MESSAGE_INPUT_ID))
                .on_input(Message::InputChanged)
                .on_submit(Message::SendMessage)
                .padding(10)
                .width(Fill);

            let send_btn = button(text("Send").size(14))
                .on_press(Message::SendMessage)
                .padding(10);

            let input_row = row![input, send_btn].spacing(8).width(Fill);

            column![topic_label, message_area, input_row]
                .spacing(8)
                .width(Fill)
                .height(Fill)
                .padding(8)
                .into()
        }
        else
        {
            container(text("Select a room").size(16))
                .center(Fill)
                .width(Fill)
                .height(Fill)
                .into()
        };

        // Member list (right sidebar).
        let member_list: Element<'_, Message> = if let Some(index) = self.active_room
        {
            let room = &self.rooms[index];
            let member_count = room.users.len();
            let members: Vec<Element<'_, Message>> = room.users.iter().map(|u|
            {
                text(&u.name).size(14).into()
            }).collect();

            container(
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
            .into()
        }
        else
        {
            container(text(""))
                .width(Length::Fixed(160.0))
                .height(Fill)
                .into()
        };

        // Main layout.
        return row![room_list, center, member_list]
            .spacing(0)
            .height(Fill)
            .width(Fill)
            .into();
    }

    fn subscription(&self) -> iced::Subscription<Message>
    {
        iced::Subscription::none()
    }

    fn theme(&self) -> Theme
    {
        Theme::Nord
    }
}
