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
    // Index of the first message in the history the server replays after a
    // reconnect re-join. These messages can repeat ones already in `messages`,
    // so a divider here separates the old chat from the replayed "new history".
    // Cleared once the user navigates away from the room.
    pub history_marker: Option<usize>,
}
