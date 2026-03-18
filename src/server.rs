use crate::room::Room;

pub struct Server
{
    pub jid: String,
    pub rooms: Vec<Room>,
}

impl Server
{
    pub fn domain(&self) -> &str
    {
        self.jid.split('@').nth(1).unwrap_or(&self.jid)
    }
}
