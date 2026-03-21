use iced::futures::SinkExt;
use iced::stream;
use log::{ debug, error };
use std::sync::{ Arc, Mutex };
use xmpp::jid::BareJid;
use xmpp::{ ClientBuilder, ClientFeature, ClientType, Event };
use xmpp::parsers::message::MessageType;

#[derive(Debug)]
pub enum XmppCommand
{
    JoinRoom(String),
    SendRoomMessage { room: String, body: String },
}

/// A cloneable handle to the command receiver, used as part of the subscription key.
/// Equality and hashing are by pointer identity so the same channel maps to the same subscription.
#[derive(Clone)]
pub struct CommandChannel(Arc<Mutex<Option<tokio::sync::mpsc::Receiver<XmppCommand>>>>);

impl PartialEq for CommandChannel
{
    fn eq(&self, other: &Self) -> bool
    {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for CommandChannel {}

impl std::hash::Hash for CommandChannel
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H)
    {
        (Arc::as_ptr(&self.0) as usize).hash(state);
    }
}

pub fn new_command_channel() -> (tokio::sync::mpsc::Sender<XmppCommand>, CommandChannel)
{
    let (tx, rx) = tokio::sync::mpsc::channel(100);
    (tx, CommandChannel(Arc::new(Mutex::new(Some(rx)))))
}

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

pub fn connect(jid: String, password: String, cmd: CommandChannel) -> impl iced::futures::Stream<Item = XmppEvent>
{
    stream::channel(100, async move |mut output|
    {
        let bare_jid = match BareJid::new(&jid)
        {
            Ok(j) => j,
            Err(e) =>
            {
                error!("Invalid JID '{}': {}", jid, e);
                let _ = output.send(XmppEvent::Disconnected(format!("Invalid JID: {}", e))).await;
                return;
            }
        };

        let cmd_rx_opt = cmd.0.lock().unwrap().take();
        let mut cmd_rx = match cmd_rx_opt
        {
            Some(rx) => rx,
            None =>
            {
                error!("Command channel receiver already consumed");
                let _ = output.send(XmppEvent::Disconnected("Internal error: command channel already consumed.".to_string())).await;
                return;
            }
        };

        let (tx, mut rx) = tokio::sync::mpsc::channel::<XmppEvent>(100);

        std::thread::spawn(move ||
        {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

            rt.block_on(async move
            {
                let nick = bare_jid.node().map(|n| n.as_str()).unwrap_or("user").to_string();

                let mut client = ClientBuilder::new(bare_jid, &password)
                    .set_client(ClientType::Pc, concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")))
                    .set_default_nick(&nick)
                    .enable_feature(ClientFeature::JoinRooms)
                    .build();

                loop
                {
                    tokio::select!
                    {
                        events = client.wait_for_events() =>
                        {
                            match events
                            {
                                Some(events) =>
                                {
                                    for event in events
                                    {
                                        debug!("XMPP event: {:?}", event);
                                        let xmpp_event = match event
                                        {
                                            Event::Online => Some(XmppEvent::Connected),
                                            Event::Disconnected(e) =>
                                            {
                                                debug!("XMPP disconnected: {}", e);
                                                let _ = tx.send(XmppEvent::Disconnected(format!("{}", e))).await;
                                                return;
                                            }
                                            Event::RoomJoined(jid) => Some(XmppEvent::RoomJoined(jid.to_string())),
                                            Event::RoomLeft(jid) => Some(XmppEvent::RoomLeft(jid.to_string())),
                                            Event::RoomMessage(_id, room, nick, body, time_info) =>
                                            {
                                                let timestamp = time_info.delays.first()
                                                    .map(|d| d.stamp.0.with_timezone(&chrono::Utc))
                                                    .unwrap_or(time_info.received);
                                                Some(XmppEvent::RoomMessage
                                                {
                                                    room: room.to_string(),
                                                    nick,
                                                    body: body.0,
                                                    timestamp,
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
                                                error!("XMPP event channel closed");
                                                return;
                                            }
                                        }
                                    }
                                }
                                None =>
                                {
                                    let _ = tx.send(XmppEvent::Disconnected("Connection closed.".to_string())).await;
                                    return;
                                }
                            }
                        }
                        cmd = cmd_rx.recv() =>
                        {
                            match cmd
                            {
                                Some(XmppCommand::JoinRoom(room_jid)) =>
                                {
                                    match BareJid::new(&room_jid)
                                    {
                                        Ok(bare_jid) =>
                                        {
                                            client.join_room(bare_jid, None, None, "", "").await;
                                        }
                                        Err(e) =>
                                        {
                                            error!("Invalid room JID '{}': {}", room_jid, e);
                                        }
                                    }
                                }
                                Some(XmppCommand::SendRoomMessage { room, body }) =>
                                {
                                    match BareJid::new(&room)
                                    {
                                        Ok(bare_jid) =>
                                        {
                                            client.send_message(bare_jid.into(), MessageType::Groupchat, "", &body).await;
                                        }
                                        Err(e) =>
                                        {
                                            error!("Invalid room JID '{}': {}", room, e);
                                        }
                                    }
                                }
                                None => return,
                            }
                        }
                    }
                }
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
