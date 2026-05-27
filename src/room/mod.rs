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
    // Index of the first message that arrived while this room was not being watched.
    pub read_marker: Option<usize>,
}
