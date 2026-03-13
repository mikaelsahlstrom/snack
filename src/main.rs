use iced::{ Application, Element, Program, Task, Theme, Fill };
use iced::widget::{ row, Id };

mod room;
mod ui;

pub const MESSAGE_SCROLL_ID: &str = "message_scroll";
pub const MESSAGE_INPUT_ID: &str = "message_input";
pub const JOIN_INPUT_ID: &str = "join_input";

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

pub struct Snack
{
    pub(crate) rooms: Vec<room::Room>,
    pub(crate) active_room: Option<usize>,
    pub(crate) message_input: String,
    pub(crate) show_join_panel: bool,
    pub(crate) join_input: String,
}

#[derive(Debug, Clone)]
pub enum Message
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
        let room_list = ui::sidebar::view(self);
        let center = ui::chat::view(self);
        let member_list = ui::members::view(self);

        row![room_list, center, member_list]
            .spacing(0)
            .height(Fill)
            .width(Fill)
            .into()
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
