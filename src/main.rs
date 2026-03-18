use iced::{ Application, Element, Program, Task, Theme, Fill };
use iced::widget::{ row, Id };

mod room;
mod ui;

pub const MESSAGE_SCROLL_ID: &str = "message_scroll";
pub const MESSAGE_INPUT_ID: &str = "message_input";
pub const JOIN_INPUT_ID: &str = "join_input";
pub const ACCOUNT_JID_INPUT_ID: &str = "account_jid_input";
pub const ACCOUNT_PASSWORD_INPUT_ID: &str = "account_password_input";

fn focus_jid_input() -> Task<Message>
{
    iced::widget::operation::focus(Id::new(ACCOUNT_JID_INPUT_ID))
}

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

#[derive(Debug, Clone, PartialEq)]
pub enum AppState
{
    Login,
    Connected,
}

pub struct Snack
{
    pub(crate) state: AppState,
    pub(crate) jid_input: String,
    pub(crate) password_input: String,
    pub(crate) connected_jid: Option<String>,
    pub(crate) connect_error: Option<String>,
    pub(crate) rooms: Vec<room::Room>,
    pub(crate) active_room: Option<usize>,
    pub(crate) message_input: String,
    pub(crate) show_join_panel: bool,
    pub(crate) join_input: String,
}

#[derive(Debug, Clone)]
pub enum Message
{
    Ignore,
    TabPressed,
    ShiftTabPressed,
    JidInputChanged(String),
    PasswordInputChanged(String),
    FocusPassword,
    Connect,
    Disconnect,
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
        (Self
        {
            state: AppState::Login,
            jid_input: String::new(),
            password_input: String::new(),
            connected_jid: None,
            connect_error: None,
            rooms: Vec::new(),
            active_room: None,
            message_input: String::new(),
            show_join_panel: false,
            join_input: String::new(),
        }, focus_jid_input())
    }

    fn title(&self) -> String
    {
        if let Some(ref jid) = self.connected_jid
        {
            return format!("Snack — {}", jid);
        }
        return "Snack".to_string();
    }

    fn update(&mut self, message: Message) -> Task<Message>
    {
        match message
        {
            Message::Ignore => {}
            Message::TabPressed =>
            {
                return iced::widget::operation::focus_next();
            }
            Message::ShiftTabPressed =>
            {
                return iced::widget::operation::focus_previous();
            }
            Message::JidInputChanged(value) =>
            {
                self.jid_input = value;
            }
            Message::PasswordInputChanged(value) =>
            {
                self.password_input = value;
            }
            Message::FocusPassword =>
            {
                return iced::widget::operation::focus(Id::new(ACCOUNT_PASSWORD_INPUT_ID));
            }
            Message::Connect =>
            {
                let jid = self.jid_input.trim().to_string();
                let password = self.password_input.trim().to_string();
                if jid.is_empty() || password.is_empty()
                {
                    self.connect_error = Some("JID and password are required.".to_string());
                    return Task::none();
                }
                if !jid.contains('@')
                {
                    self.connect_error = Some("JID must be in the format user@domain.".to_string());
                    return Task::none();
                }
                self.connected_jid = Some(jid);
                self.connect_error = None;
                self.password_input.clear();
                self.state = AppState::Connected;
                return focus_input();
            }
            Message::Disconnect =>
            {
                self.state = AppState::Login;
                self.connected_jid = None;
                self.rooms.clear();
                self.active_room = None;
                self.message_input.clear();
                self.show_join_panel = false;
                self.join_input.clear();
                return focus_jid_input();
            }
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
        match self.state
        {
            AppState::Login =>
            {
                return ui::account::view(self);
            }
            AppState::Connected =>
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
        }
    }

    fn subscription(&self) -> iced::Subscription<Message>
    {
        iced::keyboard::listen().map(|event|
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
            }
            Message::Ignore
        })
    }

    fn theme(&self) -> Theme
    {
        Theme::Nord
    }
}
