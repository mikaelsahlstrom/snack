pub mod user;
pub mod message;

pub struct Room
{
    pub jid: String,
    pub title: String,
    pub topic: String,
    pub users: Vec<user::User>,
    pub messages: Vec<message::Message>,
}
