use chrono::{ DateTime, Utc };

pub struct Message
{
    pub from: String,
    pub body: String,
    pub received: DateTime<Utc>
}
