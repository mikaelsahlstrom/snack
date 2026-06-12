use iced::widget::text_editor;

use crate::xmpp;

#[derive(Debug, Clone)]
pub enum Message
{
    Ignore,
    TabPressed,
    ShiftTabPressed,
    NextSelection,
    PrevSelection,
    JidInputChanged(String),
    PasswordInputChanged(String),
    RememberMeToggled(bool),
    SaveRoomToggled(bool),
    FocusPassword,
    Connect,
    // User-triggered immediate reconnect: skips the remaining backoff delay
    // while the worker is retrying after a dropped connection.
    ForceReconnect,
    CancelConnect,
    XmppEvent(xmpp::XmppEvent),
    Disconnect,
    SelectRoom(usize),
    SelectChat(usize),
    StartChat(String),
    InputAction(text_editor::Action),
    SendMessage,
    // Delayed checks on an optimistically-shown outgoing room message (keyed by
    // its negative temp id): show the "sending…" badge once the grace period
    // passes, and mark it failed if no server echo confirms it before the timeout.
    MarkSendPending { conversation: String, temp_id: i64 },
    MarkSendFailed { conversation: String, temp_id: i64 },
    ShowJoinPanel,
    HideJoinPanel,
    JoinInputChanged(String),
    JoinRoom,
    DismissJoinError,
    LeaveRoom,
    CloseChat,
    LeaveSelection,
    OpenUrl(String),
    ForgetAutoLogin,
    WindowFocused,
    WindowUnfocused,
}
