#[cfg(feature = "rustls-any-backend")]
use xmpp::tokio_xmpp::rustls;
use xmpp::{
    Agent, ClientBuilder, ClientFeature, ClientType, Event, RoomNick,
    jid::BareJid,
};

pub struct XmppClient
{
    agent: Agent,
}

impl XmppClient
{
    pub async fn new(jid: &str, password: &str) -> Result<Self, Err<E>>
    {
        let mut client = ClientBuilder::new(jid, password)
            .set_client_type(ClientType::Pc, concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")))
            .set_features(vec![ClientFeature::Muc])
            .build();

        Ok(Self { agent: client })
    }

    pub async fn connect(jid: &str, password: &str) -> anyhow::Result<Self>
    {
        let client = ClientBuilder::new(jid, password)
            .set_client_type(ClientType::Full, concat!("snack/", env!("CARGO_PKG_VERSION")))
            .set_features(vec![ClientFeature::Muc])
            .build()
            .await?;

        Ok(Self { agent: client })
    }

    pub async fn join_room(&self, room_jid: &str, nickname: &str) -> anyhow::Result<()>
    {
        let room_jid = BareJid::parse(room_jid)?;
        let settings = JoinRoomSettings {
            nickname: RoomNick::new(nickname.to_string()),
            message_settings: RoomMessageSettings::default(),
        };
        self.agent.join_room(room_jid, settings).await?;
        Ok(())
    }
}
