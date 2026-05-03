use crate::room::message;

pub struct Chat
{
    pub jid: String,
    pub title: String,
    pub messages: Vec<message::Message>,
    pub unread: bool,
    /// Index of the first message that arrived while this chat was not being watched.
    /// Mirrors `Room::read_marker` semantics.
    pub read_marker: Option<usize>,
}
