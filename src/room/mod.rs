pub mod user;
pub mod message;

pub struct Room
{
    pub jid: String,
    pub title: String,
    pub topic: String,
    pub users: Vec<user::User>,
    pub messages: Vec<message::Message>,
    pub unread: bool,
    /// Index of the first message received after the window lost focus.
    /// None means no divider should be shown.
    pub read_marker: Option<usize>,
}
