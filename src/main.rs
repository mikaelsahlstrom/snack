use iced::{ Application, Element, Program, Task, Theme, Fill };
use iced::widget::{ row, Id };
use log::error;

mod room;
mod ui;
mod xmpp;

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
    Connecting,
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
    pub(crate) joining_room: Option<String>,
    pub(crate) join_error: Option<String>,
    pub(crate) join_input: String,
    pub(crate) xmpp_cmd_tx: Option<tokio::sync::mpsc::Sender<xmpp::XmppCommand>>,
    pub(crate) xmpp_cmd_rx: Option<xmpp::CommandChannel>,
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
    XmppEvent(xmpp::XmppEvent),
    Disconnect,
    SelectRoom(usize),
    InputChanged(String),
    SendMessage,
    ShowJoinPanel,
    HideJoinPanel,
    JoinInputChanged(String),
    JoinRoom,
    DismissJoinError,
    LeaveRoom,
    OpenUrl(String),
    WindowFocused,
    WindowUnfocused,
}

fn main() -> iced::Result
{
    env_logger::init();
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
        return (Self
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
            joining_room: None,
            join_error: None,
            join_input: String::new(),
            xmpp_cmd_tx: None,
            xmpp_cmd_rx: None,
        }, focus_jid_input());
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
                let password = self.password_input.clone();

                if jid.is_empty() || password.is_empty()
                {
                    error!("Connection failed: JID and password are required");
                    self.connect_error = Some("JID and password are required.".to_string());

                    return Task::none();
                }

                if !jid.contains('@')
                {
                    error!("Connection failed: invalid JID format '{}'", jid);
                    self.connect_error = Some("JID must be in the format user@domain.".to_string());

                    return Task::none();
                }

                self.connected_jid = Some(jid.clone());
                self.connect_error = None;

                let (cmd_tx, cmd_rx) = xmpp::new_command_channel(jid, password);
                self.xmpp_cmd_tx = Some(cmd_tx);
                self.xmpp_cmd_rx = Some(cmd_rx);

                self.state = AppState::Connecting;

                return Task::none();
            }
            Message::XmppEvent(event) =>
            {
                log::debug!("UI received XmppEvent: {:?}", event);
                match event
                {
                    xmpp::XmppEvent::Connected =>
                    {
                        self.password_input.clear();
                        self.state = AppState::Connected;

                        return focus_join_input();
                    }
                    xmpp::XmppEvent::Disconnected(reason) =>
                    {
                        error!("Disconnected: {}", reason);

                        self.connect_error = Some(reason);
                        self.connected_jid = None;
                        self.state = AppState::Login;
                        self.rooms.clear();
                        self.active_room = None;
                        self.message_input.clear();
                        self.show_join_panel = false;
                        self.joining_room = None;
                        self.join_error = None;
                        self.join_input.clear();
                        self.xmpp_cmd_tx = None;
                        self.xmpp_cmd_rx = None;

                        return focus_jid_input();
                    }
                    xmpp::XmppEvent::RoomJoined { room: jid, members } =>
                    {
                        self.joining_room = None;
                        self.join_error = None;

                        if let Some(pos) = self.rooms.iter().position(|r| r.jid == jid)
                        {
                            self.active_room = Some(pos);
                            self.rooms[pos].users = members.into_iter().map(|m| room::user::User
                            {
                                jid: m.jid,
                                name: m.nick,
                                show: m.show,
                                status: m.status,
                            }).collect();
                        }
                        else
                        {
                            let title = jid.split('@').next().unwrap_or(&jid).to_string();
                            let users = members.into_iter().map(|m| room::user::User
                            {
                                jid: m.jid,
                                name: m.nick,
                                show: m.show,
                                status: m.status,
                            }).collect();
                            self.rooms.push(room::Room
                            {
                                jid,
                                title,
                                topic: String::new(),
                                users,
                                messages: Vec::new(),
                                unread: false,
                                read_marker: None,
                            });
                            self.active_room = Some(self.rooms.len() - 1);
                        }

                        self.show_join_panel = false;
                        return Task::batch([snap_to_bottom(), focus_input()]);
                    }
                    xmpp::XmppEvent::RoomJoinFailed { room: _, reason } =>
                    {
                        self.joining_room = None;
                        self.join_error = Some(reason);

                        return focus_join_input();
                    }
                    xmpp::XmppEvent::PresenceError { from, condition, text } =>
                    {
                        let is_join_error = self.joining_room.as_ref().map_or(false, |room|
                        {
                            from == *room || from.starts_with(&format!("{}/", room))
                        });

                        if is_join_error
                        {
                            let message = match condition.as_str()
                            {
                                "item-not-found" => "Room does not exist.".to_string(),
                                "not-allowed" => "Not allowed to join this room.".to_string(),
                                "forbidden" => "You are banned from this room.".to_string(),
                                "conflict" => "Nickname is already in use.".to_string(),
                                "service-unavailable" => "Room service is unavailable.".to_string(),
                                "registration-required" => "Registration is required to join this room.".to_string(),
                                "not-authorized" => "Not authorized to join this room.".to_string(),
                                _ => text.unwrap_or_else(|| format!("Could not join room: {}.", condition)),
                            };

                            self.joining_room = None;
                            self.join_error = Some(message);

                            return focus_join_input();
                        }
                    }
                    xmpp::XmppEvent::RoomLeft(jid) =>
                    {
                        if let Some(pos) = self.rooms.iter().position(|r| r.jid == jid)
                        {
                            self.rooms.remove(pos);
                            if self.active_room == Some(pos)
                            {
                                self.active_room = None;
                            }
                            else if let Some(active) = self.active_room
                            {
                                if active > pos
                                {
                                    self.active_room = Some(active - 1);
                                }
                            }
                        }
                    }
                    xmpp::XmppEvent::MemberJoined { room, member } =>
                    {
                        if let Some(r) = self.rooms.iter_mut().find(|r| r.jid == room)
                        {
                            let jid = member.jid.clone();
                            let nick = member.nick.clone();
                            let existing_idx = r.users.iter().position(|u| u.name == nick);

                            if let Some(idx) = existing_idx
                            {
                                let old_show = r.users[idx].show.clone();
                                r.users[idx].show = member.show.clone();
                                r.users[idx].status = member.status;

                                if old_show != r.users[idx].show
                                {
                                    r.messages.push(room::message::Message::Event
                                    {
                                        kind: room::message::EventKind::StatusChanged(r.users[idx].show.clone()),
                                        nick,
                                        received: chrono::Utc::now(),
                                    });
                                }
                            }
                            else
                            {
                                r.users.push(room::user::User
                                {
                                    jid: jid.clone(),
                                    name: nick.clone(),
                                    show: member.show,
                                    status: member.status,
                                });
                                r.messages.push(room::message::Message::Event
                                {
                                    kind: room::message::EventKind::Joined,
                                    nick,
                                    received: chrono::Utc::now(),
                                });
                            }
                        }
                    }
                    xmpp::XmppEvent::MemberLeft { room, nick } =>
                    {
                        if let Some(r) = self.rooms.iter_mut().find(|r| r.jid == room)
                        {
                            r.users.retain(|u| u.name != nick);
                            r.messages.push(room::message::Message::Event
                            {
                                kind: room::message::EventKind::Left,
                                nick,
                                received: chrono::Utc::now(),
                            });
                        }
                    }
                    xmpp::XmppEvent::RoomMessage { room, nick, body, timestamp } =>
                    {
                        let room_idx = self.rooms.iter().position(|r| r.jid == room);
                        if let Some(idx) = room_idx
                        {
                            self.rooms[idx].messages.push(room::message::Message::Chat
                            {
                                from: nick,
                                body,
                                received: timestamp,
                            });

                            let is_active = self.active_room == Some(idx);

                            if !is_active
                            {
                                self.rooms[idx].unread = true;
                            }

                            return snap_to_bottom();
                        }
                    }
                    xmpp::XmppEvent::RoomSubject { room, subject } =>
                    {
                        if let Some(r) = self.rooms.iter_mut().find(|r| r.jid == room)
                        {
                            r.topic = subject;
                        }
                    }
                }
            }
            Message::Disconnect =>
            {
                self.state = AppState::Login;
                self.connected_jid = None;
                self.rooms.clear();
                self.active_room = None;
                self.message_input.clear();
                self.show_join_panel = false;
                self.joining_room = None;
                self.join_error = None;
                self.join_input.clear();
                self.xmpp_cmd_tx = None;
                self.xmpp_cmd_rx = None;

                return focus_jid_input();
            }
            Message::SelectRoom(index) =>
            {
                // Stamp the room we're leaving so messages arriving while away show as new.
                if let Some(old_idx) = self.active_room
                {
                    if old_idx != index
                    {
                        let len = self.rooms[old_idx].messages.len();
                        self.rooms[old_idx].read_marker = Some(len);
                    }
                }

                self.active_room = Some(index);
                self.show_join_panel = false;
                if let Some(r) = self.rooms.get_mut(index)
                {
                    r.unread = false;
                    r.read_marker = None;
                }

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
                        if let Some(ref tx) = self.xmpp_cmd_tx
                        {
                            let room_jid = self.rooms[index].jid.clone();
                            if tx.try_send(xmpp::XmppCommand::SendRoomMessage
                            {
                                room: room_jid,
                                body: body.clone(),
                            }).is_err()
                            {
                                return focus_input();
                            }
                        }

                        self.message_input.clear();

                        return Task::batch([snap_to_bottom(), focus_input()]);
                    }
                }
            }
            Message::ShowJoinPanel =>
            {
                self.show_join_panel = true;
                self.join_input.clear();
                self.join_error = None;
                self.joining_room = None;

                return focus_join_input();
            }
            Message::HideJoinPanel =>
            {
                self.show_join_panel = false;
                self.join_error = None;

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
                    // If already in this room, just switch to it.
                    if let Some(pos) = self.rooms.iter().position(|r| r.jid == jid)
                    {
                        self.active_room = Some(pos);
                        self.show_join_panel = false;
                        self.join_input.clear();
                        self.join_error = None;

                        return Task::batch([snap_to_bottom(), focus_input()]);
                    }

                    if let Some(ref tx) = self.xmpp_cmd_tx
                    {
                        let _ = tx.try_send(xmpp::XmppCommand::JoinRoom(jid.clone()));
                    }

                    self.joining_room = Some(jid);
                    self.join_error = None;
                }
            }
            Message::DismissJoinError =>
            {
                self.join_error = None;
                return focus_join_input();
            }
            Message::LeaveRoom =>
            {
                if let Some(index) = self.active_room
                {
                    let room_jid = self.rooms[index].jid.clone();

                    if let Some(ref tx) = self.xmpp_cmd_tx
                    {
                        // Derive nick the same way the XMPP thread does.
                        let nick = self.connected_jid
                            .as_deref()
                            .and_then(|j| j.split('@').next())
                            .unwrap_or("user")
                            .to_string();

                        let _ = tx.try_send(xmpp::XmppCommand::LeaveRoom
                        {
                            room: room_jid,
                            nick,
                        });
                    }

                    // Room removal is driven by XmppEvent::RoomLeft.
                }
            }
            Message::OpenUrl(url) =>
            {
                if let Err(e) = open::that(&url)
                {
                    error!("Failed to open URL {}: {}", url, e);
                }
            }
            Message::WindowUnfocused =>
            {
                for room in self.rooms.iter_mut()
                {
                    room.read_marker = Some(room.messages.len());
                }
            }
            Message::WindowFocused => {}
        }

        return Task::none();
    }

    fn view(&self) -> Element<'_, Message>
    {
        match self.state
        {
            AppState::Login | AppState::Connecting =>
            {
                return ui::account::view(self);
            }
            AppState::Connected =>
            {
                let room_list = ui::sidebar::view(self);
                let center = ui::chat::view(self);
                let member_list = ui::members::view(self);

                return row![room_list, center, member_list]
                    .spacing(0)
                    .height(Fill)
                    .width(Fill)
                    .into();
            }
        }
    }

    fn subscription(&self) -> iced::Subscription<Message>
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

    fn theme(&self) -> Theme
    {
        return Theme::Nord;
    }
}
