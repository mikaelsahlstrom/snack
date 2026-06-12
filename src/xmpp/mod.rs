use iced::futures::SinkExt;
use iced::stream;
use log::error;
use std::sync::{ Arc, Mutex };
use std::time::Duration;

/// Delay before the first reconnect attempt after a session drops.
const INITIAL_BACKOFF: Duration = Duration::from_secs(1);
/// Upper bound on the exponential backoff between reconnect attempts.
const MAX_BACKOFF: Duration = Duration::from_secs(60);
/// How often to send a keep-alive ping on an established session. This is what
/// surfaces a silently half-open socket, where reads would otherwise block
/// forever and never report the drop.
const PING_INTERVAL: Duration = Duration::from_secs(30);
/// How long to wait for a ping reply before treating the connection as dead.
const PING_TIMEOUT: Duration = Duration::from_secs(10);
/// Upper bound on a single connection-setup attempt (DNS, TCP, TLS, SASL,
/// resource bind). Without this a reconnect fired before the network is back
/// (e.g. right after wake-from-sleep) blocks forever on a dead socket.
const SETUP_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug)]
pub enum XmppCommand
{
    JoinRoom(String),
    LeaveRoom { room: String, nick: String },
    SendRoomMessage { room: String, body: String },
    SendDirectMessage { to: String, body: String },
}

struct ChannelInner
{
    rx: Option<tokio::sync::mpsc::Receiver<XmppCommand>>,
    jid: String,
    password: String,
}

#[derive(Clone)]
pub struct CommandChannel(Arc<Mutex<ChannelInner>>);

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

pub fn new_command_channel(jid: String, password: String) -> (tokio::sync::mpsc::Sender<XmppCommand>, CommandChannel)
{
    let (tx, rx) = tokio::sync::mpsc::channel(100);
    (tx, CommandChannel(Arc::new(Mutex::new(ChannelInner { rx: Some(rx), jid, password }))))
}

#[derive(Debug, Clone)]
pub enum XmppEvent
{
    Connected,
    /// An established session dropped; the worker is now retrying with backoff.
    Reconnecting,
    /// A dropped session was re-established. The UI re-joins its open rooms.
    Reconnected,
    Disconnected(String),
    RoomJoined { room: String, members: Vec<::xmpp::RoomMember> },
    RoomJoinFailed { room: String, reason: String },
    RoomLeft(String),
    MemberJoined { room: String, member: ::xmpp::RoomMember },
    MemberLeft { room: String, nick: String },
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
    PresenceError
    {
        from: String,
        condition: String,
        text: Option<String>,
    },
    DirectMessage
    {
        from: String,
        body: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
}

/// Why a running session ended, telling the outer loop whether to retry.
enum SessionOutcome
{
    /// The connection dropped (read error, server close, or a failed
    /// keep-alive ping). The outer loop reconnects with backoff.
    Dropped,
    /// The UI logged out: the command sender was dropped. Stop for good.
    LoggedOut,
    /// The UI (and the event stream) went away. Stop for good.
    UiGone,
}

/// True for failures that must never be retried: bad credentials or no
/// usable SASL mechanism. The user has to re-authenticate.
fn is_auth_failure(e: &::xmpp::XmppError) -> bool
{
    matches!(e, ::xmpp::XmppError::Auth(_) | ::xmpp::XmppError::NoSaslMechanism)
}

/// Drain commands issued while reconnecting (they can't be delivered, so they
/// are discarded) and complete only once the channel closes, i.e. on logout.
async fn drain_until_closed(cmd_rx: &mut tokio::sync::mpsc::Receiver<XmppCommand>)
{
    while let Some(cmd) = cmd_rx.recv().await
    {
        log::debug!("Dropping command issued during reconnect backoff: {:?}", cmd);
    }
}

/// Run one established session until it ends. Multiplexes server events, UI
/// commands, and a keep-alive ping that detects a silently half-open socket.
/// `established` distinguishes a fresh login from a reconnect so the server's
/// `Connected` event maps to the right UI event.
async fn run_session(
    mut client: ::xmpp::XmppClient,
    mut event_rx: tokio::sync::mpsc::Receiver<::xmpp::XmppEvent>,
    cmd_rx: &mut tokio::sync::mpsc::Receiver<XmppCommand>,
    bridge_tx: &tokio::sync::mpsc::Sender<XmppEvent>,
    nick: &str,
    established: bool,
) -> SessionOutcome
{
    let mut ping = tokio::time::interval(PING_INTERVAL);
    ping.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    // The first tick fires immediately; skip it so we don't ping right away.
    ping.tick().await;

    let outcome = loop
    {
        tokio::select!
        {
            event = event_rx.recv() =>
            {
                match event
                {
                    Some(ev) =>
                    {
                        let mapped = match ev
                        {
                            ::xmpp::XmppEvent::Connected =>
                            {
                                // A re-established session reuses the server's
                                // Connected event, but the UI handles it differently.
                                Some(if established { XmppEvent::Reconnected } else { XmppEvent::Connected })
                            }
                            ::xmpp::XmppEvent::RoomJoined { room, members } =>
                            {
                                Some(XmppEvent::RoomJoined { room, members })
                            }
                            ::xmpp::XmppEvent::RoomLeft(room) =>
                            {
                                Some(XmppEvent::RoomLeft(room))
                            }
                            ::xmpp::XmppEvent::MemberJoined { room, member } =>
                            {
                                Some(XmppEvent::MemberJoined { room, member })
                            }
                            ::xmpp::XmppEvent::MemberLeft { room, nick } =>
                            {
                                Some(XmppEvent::MemberLeft { room, nick })
                            }
                            ::xmpp::XmppEvent::RoomMessage { room, nick, body, timestamp } =>
                            {
                                let ts = timestamp
                                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                                    .map(|dt| dt.with_timezone(&chrono::Utc))
                                    .unwrap_or_else(chrono::Utc::now);
                                Some(XmppEvent::RoomMessage { room, nick, body, timestamp: ts })
                            }
                            ::xmpp::XmppEvent::RoomSubject { room, subject } =>
                            {
                                Some(XmppEvent::RoomSubject { room, subject })
                            }
                            ::xmpp::XmppEvent::PresenceError { from, error_type: _, condition, text } =>
                            {
                                Some(XmppEvent::PresenceError { from, condition, text })
                            }
                            ::xmpp::XmppEvent::DirectMessage { from, body, timestamp } =>
                            {
                                let ts = timestamp
                                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                                    .map(|dt| dt.with_timezone(&chrono::Utc))
                                    .unwrap_or_else(chrono::Utc::now);
                                Some(XmppEvent::DirectMessage { from, body, timestamp: ts })
                            }
                            _ => None,
                        };

                        if let Some(evt) = mapped
                        {
                            if bridge_tx.send(evt).await.is_err()
                            {
                                break SessionOutcome::UiGone;
                            }
                        }
                    }
                    None => break SessionOutcome::Dropped,
                }
            }
            cmd = cmd_rx.recv() =>
            {
                match cmd
                {
                    Some(XmppCommand::JoinRoom(room_jid)) =>
                    {
                        if let Err(e) = client.join_room(&room_jid, nick).await
                        {
                            let _ = bridge_tx.send(XmppEvent::RoomJoinFailed
                            {
                                room: room_jid,
                                reason: e.to_string(),
                            }).await;
                        }
                    }
                    Some(XmppCommand::SendRoomMessage { room, body }) =>
                    {
                        if let Err(e) = client.send_room_message(&room, &body).await
                        {
                            error!("Failed to send message: {}", e);
                        }
                    }
                    Some(XmppCommand::LeaveRoom { room, nick }) =>
                    {
                        if let Err(e) = client.leave_room(&room, &nick).await
                        {
                            error!("Failed to leave room: {}", e);
                        }
                    }
                    Some(XmppCommand::SendDirectMessage { to, body }) =>
                    {
                        if let Err(e) = client.send_message(&to, &body).await
                        {
                            error!("Failed to send direct message: {}", e);
                        }
                    }
                    None => break SessionOutcome::LoggedOut,
                }
            }
            _ = ping.tick() =>
            {
                // A half-open socket never surfaces a read error, so probe it.
                if let Err(e) = client.ping(None, PING_TIMEOUT).await
                {
                    error!("Keep-alive ping failed, treating session as dropped: {}", e);
                    break SessionOutcome::Dropped;
                }
            }
        }
    };

    // Stop the (possibly read-blocked) reader task and close the socket before
    // the outer loop reconnects or exits.
    client.close().await;

    return outcome;
}

pub fn connect(cmd: CommandChannel) -> impl iced::futures::Stream<Item = XmppEvent>
{
    stream::channel(100, async move |mut output|
    {
        let (cmd_rx, jid, password) =
        {
            let mut inner = cmd.0.lock().unwrap();
            let rx = inner.rx.take();
            let jid = std::mem::take(&mut inner.jid);
            let password = std::mem::take(&mut inner.password);
            (rx, jid, password)
        };

        let mut cmd_rx = match cmd_rx
        {
            Some(rx) => rx,
            None =>
            {
                error!("Command channel receiver already consumed");
                let _ = output.send(XmppEvent::Disconnected("Internal error: command channel already consumed.".to_string())).await;

                return;
            }
        };

        let nick = jid.split('@').next().unwrap_or("user").to_string();

        // Bridge between iced's async executor and tokio: libxmpp requires a
        // tokio runtime for TCP, TLS, and spawned tasks.
        let (bridge_tx, mut bridge_rx) = tokio::sync::mpsc::channel::<XmppEvent>(100);

        std::thread::spawn(move ||
        {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

            rt.block_on(async move
            {
                // Outer reconnect loop. It owns cmd_rx and bridge_tx and keeps
                // re-establishing the underlying client without tearing down the
                // iced subscription, so the UI's rooms/chats/selection survive a
                // transient drop untouched.
                let mut backoff = INITIAL_BACKOFF;
                let mut established = false;

                loop
                {
                    // Bound the setup path with a timeout. On expiry the in-flight
                    // future is dropped, cancelling the stalled connect, and we
                    // synthesize a transient error so the existing Err arm handles
                    // it (retry when reconnecting, drop to login on first connect).
                    let result = match tokio::time::timeout(
                        SETUP_TIMEOUT,
                        ::xmpp::XmppClient::new(&jid, &password),
                    ).await
                    {
                        Ok(result) => result,
                        Err(_) => Err(::xmpp::XmppError::Timeout(
                            "connection setup timed out".to_string(),
                        )),
                    };

                    match result
                    {
                        Ok((client, event_rx)) =>
                        {
                            backoff = INITIAL_BACKOFF;

                            let outcome = run_session(client, event_rx, &mut cmd_rx, &bridge_tx, &nick, established).await;
                            established = true;

                            match outcome
                            {
                                // Logout or the UI going away: stop for good.
                                SessionOutcome::LoggedOut | SessionOutcome::UiGone => return,
                                // The session dropped: fall through to backoff and retry.
                                SessionOutcome::Dropped => {}
                            }
                        }
                        Err(e) =>
                        {
                            // The initial connect failing (auth or transient) keeps the
                            // old behavior of dropping to the login screen. Auth failures
                            // are never retried, even mid-session.
                            if !established || is_auth_failure(&e)
                            {
                                let _ = bridge_tx.send(XmppEvent::Disconnected(e.to_string())).await;
                                return;
                            }
                            // A transient connect failure while reconnecting: keep retrying.
                        }
                    }

                    let _ = bridge_tx.send(XmppEvent::Reconnecting).await;

                    // Wait out the backoff, but abort immediately if the UI logs
                    // out (which drops the command sender and closes cmd_rx).
                    tokio::select!
                    {
                        _ = tokio::time::sleep(backoff) => {}
                        _ = drain_until_closed(&mut cmd_rx) => return,
                    }

                    backoff = (backoff * 2).min(MAX_BACKOFF);
                }
            });
        });

        while let Some(event) = bridge_rx.recv().await
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
