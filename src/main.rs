use iced::{ Application, Element, Program, Task, Theme, Length, Fill };
use iced::widget::{ column, container, row, scrollable, text, text_input, button, Id };

mod room;

const MESSAGE_SCROLL_ID: &str = "message_scroll";

fn snap_to_bottom() -> Task<Message>
{
    iced::widget::operation::snap_to_end(Id::new(MESSAGE_SCROLL_ID))
}

struct Snack
{
    rooms: Vec<room::Room>,
    active_room: Option<usize>,
    message_input: String,
}

#[derive(Debug, Clone)]
enum Message
{
    SelectRoom(usize),
    InputChanged(String),
    SendMessage,
}

fn main() -> iced::Result
{
    return application().run();
}

fn application() -> Application<impl Program<Message = Message, Theme = Theme>>
{
    return iced::application(Snack::new, Snack::update, Snack::view)
                .subscription(Snack::subscription)
                .title(Snack::title);
}

impl Snack
{
    fn new() -> Self
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
                    room::message::Message { from: "Alice".into(), body: "Hello everyone!".into(), time: "10:00".into() },
                    room::message::Message { from: "Bob".into(), body: "Hey Alice!".into(), time: "10:01".into() },
                ],
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
                    room::message::Message { from: "Charlie".into(), body: "Welcome to XMPP!".into(), time: "09:30".into() },
                ],
            },
        ];

        Self
        {
            rooms,
            active_room: Some(0),
            message_input: String::new(),
        }
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
                return snap_to_bottom();
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
                            time: "now".to_string(),
                        });

                        self.message_input.clear();

                        return snap_to_bottom();
                    }
                }
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message>
    {
        // --- Room list (left sidebar) ---
        let room_list: Element<'_, Message> =
        {
            let items: Vec<Element<'_, Message>> = self.rooms.iter().enumerate().map(|(i, r)|
            {
                let is_active = self.active_room == Some(i);
                let label = text(&r.title).size(16);
                let btn = button(label)
                    .on_press(Message::SelectRoom(i))
                    .width(Fill)
                    .padding(10);

                let btn: Element<'_, Message> = if is_active
                {
                    btn.style(button::primary).into()
                }
                else
                {
                    btn.style(button::secondary).into()
                };

                return btn;
            }).collect();

            container(
                scrollable(
                    column(items).spacing(4).width(Fill)
                )
            )
            .width(Length::Fixed(200.0))
            .height(Fill)
            .padding(8)
            .style(container::bordered_box)
            .into()
        };

        // --- Center: messages + input ---
        let center: Element<'_, Message> = if let Some(index) = self.active_room
        {
            let room = &self.rooms[index];

            // Room topic
            let topic_label = container(text(&room.topic).size(14))
                .padding(8)
                .width(Fill)
                .style(container::bordered_box);

            // Message list
            let messages: Vec<Element<'_, Message>> = room.messages.iter().map(|m|
            {
                let line = text(format!("[{}] {}: {}", m.time, m.from, m.body)).size(14);
                container(line).padding(4).width(Fill).into()
            }).collect();

            let message_area = scrollable(
                column(messages).spacing(2).width(Fill)
            )
            .id(Id::new(MESSAGE_SCROLL_ID))
            .height(Fill)
            .width(Fill);

            // Input bar
            let input = text_input("Type a message...", &self.message_input)
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

        // --- Member list (right sidebar) ---
        let member_list: Element<'_, Message> = if let Some(index) = self.active_room
        {
            let room = &self.rooms[index];
            let header = text("Members").size(14);
            let members: Vec<Element<'_, Message>> = room.users.iter().map(|u|
            {
                text(&u.name).size(14).into()
            }).collect();

            let mut items = vec![header.into()];
            items.extend(members);

            container(
                scrollable(
                    column(items).spacing(6).width(Fill)
                )
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

        // --- Main layout ---
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
}
