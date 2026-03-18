use iced::futures::SinkExt;
use iced::stream;
use log::debug;
use xmpp::jid::BareJid;
use xmpp::{ ClientBuilder, ClientFeature, ClientType, Event };

#[derive(Debug, Clone)]
pub enum XmppEvent
{
    Connected,
    Disconnected(String),
    RoomJoined(String),
    RoomLeft(String),
    RoomMessage
    {
        room: String,
        nick: String,
        body: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    RoomSubject
    {
        room: String,
        subject: String,
    },
}

pub fn connect(jid: String, password: String) -> impl iced::futures::Stream<Item = XmppEvent>
{
    stream::channel(100, async move |mut output|
    {
        let bare_jid = match BareJid::new(&jid)
        {
            Ok(j) => j,
            Err(e) =>
            {
                let _ = output.send(XmppEvent::Disconnected(format!("Invalid JID: {}", e))).await;
                return;
            }
        };

        let (tx, mut rx) = tokio::sync::mpsc::channel::<XmppEvent>(100);

        std::thread::spawn(move ||
        {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
            rt.block_on(async move
            {
                let mut agent = ClientBuilder::new(bare_jid, &password)
                    .set_client(ClientType::Pc, "snack")
                    .enable_feature(ClientFeature::JoinRooms)
                    .build();

                while let Some(events) = agent.wait_for_events().await
                {
                    for event in events
                    {
                        debug!("XMPP event: {:?}", event);
                        let xmpp_event = match event
                        {
                            Event::Online => Some(XmppEvent::Connected),
                            Event::Disconnected(e) =>
                            {
                                let _ = tx.send(XmppEvent::Disconnected(format!("{}", e))).await;
                                return;
                            }
                            Event::RoomJoined(jid) => Some(XmppEvent::RoomJoined(jid.to_string())),
                            Event::RoomLeft(jid) => Some(XmppEvent::RoomLeft(jid.to_string())),
                            Event::RoomMessage(_id, room, nick, body, time_info) =>
                            {
                                Some(XmppEvent::RoomMessage
                                {
                                    room: room.to_string(),
                                    nick,
                                    body: body.0,
                                    timestamp: time_info.received,
                                })
                            }
                            Event::RoomSubject(room, _nick, subject, _time_info) =>
                            {
                                Some(XmppEvent::RoomSubject
                                {
                                    room: room.to_string(),
                                    subject,
                                })
                            }
                            _ => None,
                        };

                        if let Some(evt) = xmpp_event
                        {
                            if tx.send(evt).await.is_err()
                            {
                                return;
                            }
                        }
                    }
                }

                let _ = tx.send(XmppEvent::Disconnected("Connection closed.".to_string())).await;
            });
        });

        while let Some(event) = rx.recv().await
        {
            let is_disconnect = matches!(&event, XmppEvent::Disconnected(_));
            let _ = output.send(event).await;
            if is_disconnect
            {
                return;
            }
        }
    })
}
