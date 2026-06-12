use chrono::{ DateTime, Utc };

pub enum EventKind
{
    Joined,
    Left,
    StatusChanged(Option<String>),
}

// Delivery state of a chat message. Most messages are `Confirmed` (received from
// the server, or a DM we echo locally) and carry nothing extra — this build keeps
// no persistent message ids. Our own *room* messages are shown optimistically the
// instant they're sent and only grow a status badge if the server echo is slow.
// The non-`Confirmed` variants carry a negative temporary id the grace/failure
// timers use to find that exact entry; the echo clears it back to `Confirmed`.
//   Sending  – just sent, within a short grace period; rendered with no badge, so a
//              normal (fast) round-trip never flickers an indicator.
//   Pending  – grace elapsed without an echo; rendered with a "sending…" badge.
//   Failed   – no echo arrived before the timeout; rendered with a "failed" badge.
//              A late echo still upgrades it back to Confirmed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatStatus
{
    Confirmed,
    Sending(i64),
    Pending(i64),
    Failed(i64),
}

pub enum Message
{
    Chat
    {
        from: String,
        body: String,
        received: DateTime<Utc>,
        status: ChatStatus,
    },
    Event
    {
        kind: EventKind,
        nick: String,
        received: DateTime<Utc>,
    },
}

pub fn mentions(body: &str, nick: &str) -> bool
{
    if nick.is_empty()
    {
        return false;
    }

    let body_lower = body.to_lowercase();
    let nick_lower = nick.to_lowercase();

    return body_lower.match_indices(&nick_lower).any(|(start, matched)|
    {
        let end = start + matched.len();
        let before_ok = start == 0
            || !body_lower.as_bytes()[start - 1].is_ascii_alphanumeric();
        let after_ok = end == body_lower.len()
            || !body_lower.as_bytes()[end].is_ascii_alphanumeric();
        return before_ok && after_ok;
    });
}
