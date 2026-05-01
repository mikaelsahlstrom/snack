use chrono::{ DateTime, Utc };

pub enum EventKind
{
    Joined,
    Left,
    StatusChanged(Option<String>),
}

pub enum Message
{
    Chat
    {
        from: String,
        body: String,
        received: DateTime<Utc>,
    },
    Event
    {
        kind: EventKind,
        nick: String,
        received: DateTime<Utc>,
    },
}
