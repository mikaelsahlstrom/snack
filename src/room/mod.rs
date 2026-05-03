pub mod user;
pub mod message;
pub mod chat;

pub struct Room
{
    pub jid: String,
    pub title: String,
    pub topic: String,
    pub users: Vec<user::User>,
    pub messages: Vec<message::Message>,
    pub unread: bool,
    /// Index of the first message that arrived while this room was not being watched.
    /// Set when the window loses focus (for the active room) or when the user switches
    /// away from this room. Cleared when the user switches to this room or the window
    /// regains focus while this room is active.
    /// `None` means no "new messages" divider should be shown.
    pub read_marker: Option<usize>,
}
