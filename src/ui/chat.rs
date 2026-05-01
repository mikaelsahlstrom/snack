use iced::{ Color, Element, Fill, Length };
use iced::widget::{ button, column, container, rich_text, row, scrollable, span, text, text_input, Id };

use crate::{ Message, Snack, MESSAGE_SCROLL_ID, MESSAGE_INPUT_ID };
use crate::room::message::{ Message as RoomMessage, EventKind };
use crate::ui::{ join, style };

/// Split text into alternating (plain, url) fragments.
fn parse_urls(body: &str) -> Vec<(&str, bool)>
{
    let mut parts = Vec::new();
    let mut remaining = body;

    while let Some(start) = remaining.find("https://").or_else(|| remaining.find("http://"))
    {
        if start > 0
        {
            parts.push((&remaining[..start], false));
        }

        let url_text = &remaining[start..];
        let end = url_text.find(|c: char| c.is_whitespace()).unwrap_or(url_text.len());

        parts.push((&remaining[start..start + end], true));
        remaining = &remaining[start + end..];
    }

    if !remaining.is_empty()
    {
        parts.push((remaining, false));
    }

    return parts;
}

pub fn view(state: &Snack) -> Element<'_, Message>
{
    if state.show_join_panel || state.joining_room.is_some() || state.join_error.is_some()
    {
        return join::view(state);
    }

    if let Some(index) = state.active_room
    {
        let room = &state.rooms[index];

        // Room topic with leave button.
        let leave_btn = button(text("Leave").size(12))
            .on_press(Message::LeaveRoom)
            .padding(4)
            .style(button::text);

        let topic_label = container(
            row![
                text(&room.topic).size(14),
                text("").width(Fill),
                leave_btn,
            ].align_y(iced::Alignment::Center).width(Fill)
        )
            .padding(8)
            .width(Fill)
            .style(container::bordered_box);

        // Message list.
        let today = chrono::Local::now().date_naive();

        // Derive local nick for mention highlighting.
        let my_nick: Option<String> = state.connected_jid
            .as_deref()
            .and_then(|j| j.split('@').next())
            .map(|n| n.to_lowercase());

        // Estimate nick column width from the longest nick in chat messages.
        let max_nick_len = room.messages.iter()
            .filter_map(|m| match m
            {
                RoomMessage::Chat { from, .. } => Some(from.len()),
                _ => None,
            })
            .max()
            .unwrap_or(4);
        // ~8px per character at size 14 + 2 chars for ": "
        let nick_width = ((max_nick_len + 2) as f32) * 8.0;

        let mut messages: Vec<Element<'_, Message>> = Vec::with_capacity(room.messages.len() + 1);

        for (i, m) in room.messages.iter().enumerate()
        {
            // Insert a "new messages" divider before the first new message.
            if room.read_marker == Some(i) && i < room.messages.len()
            {
                let accent = Color::from_rgb(0.60, 0.40, 0.40);
                let new_label = text("  new messages  ").size(11).color(accent);
                let line = || container(text(""))
                    .height(1)
                    .width(Fill)
                    .style(|_: &_| container::Style
                    {
                        background: Some(iced::Background::Color(Color::from_rgb(0.60, 0.40, 0.40))),
                        ..Default::default()
                    });
                let divider: Element<'_, Message> = row![line(), new_label, line()]
                    .align_y(iced::Alignment::Center)
                    .width(Fill)
                    .into();
                messages.push(divider);
            }

            match m
            {
                RoomMessage::Chat { from, body, received } =>
                {
                    let local_time = received.with_timezone(&chrono::Local);
                    let timestamp = if local_time.date_naive() == today
                    {
                        local_time.format("%H:%M:%S").to_string()
                    }
                    else
                    {
                        local_time.format("%Y-%m-%d %H:%M:%S").to_string()
                    };

                    let time_color = Color::from_rgb(0.40, 0.44, 0.50);
                    let nick_color = Color::from_rgb(0.60, 0.64, 0.70);

                    let time_width = if local_time.date_naive() == today { 65.0 } else { 145.0 };
                    let time_label = text(timestamp).size(14).color(time_color)
                        .width(Length::Fixed(time_width));
                    let nick_label = text(format!("{}: ", from)).size(14).color(nick_color)
                        .width(Length::Fixed(nick_width));

                    let link_color = Color::from_rgb(0.53, 0.75, 0.82);
                    let body_spans: Vec<_> = parse_urls(body).into_iter().map(|(s, is_url)|
                    {
                        if is_url
                        {
                            span(s.to_string()).color(link_color).underline(true).link(s.to_string())
                        }
                        else
                        {
                            span(s.to_string())
                        }
                    }).collect();

                    let body_label = rich_text(body_spans)
                        .on_link_click(Message::OpenUrl)
                        .size(14)
                        .width(Fill);

                    let msg_row = row![time_label, nick_label, body_label]
                        .spacing(4).width(Fill);

                    let is_mention = my_nick.as_ref()
                        .is_some_and(|nick|
                        {
                            let body_lower = body.to_lowercase();
                            body_lower.match_indices(nick.as_str()).any(|(start, matched)|
                            {
                                let end = start + matched.len();
                                let before_ok = start == 0 || !body_lower.as_bytes()[start - 1].is_ascii_alphanumeric();
                                let after_ok = end == body_lower.len() || !body_lower.as_bytes()[end].is_ascii_alphanumeric();
                                before_ok && after_ok
                            })
                        });

                    let msg_container = container(msg_row).padding(4).width(Fill);

                    let msg_element: Element<'_, Message> = if is_mention
                    {
                        msg_container.style(style::mention_highlight).into()
                    }
                    else
                    {
                        msg_container.into()
                    };

                    messages.push(msg_element);
                }
                RoomMessage::Event { kind, nick, received } =>
                {
                    let local_time = received.with_timezone(&chrono::Local);
                    let timestamp = if local_time.date_naive() == today
                    {
                        local_time.format("%H:%M:%S").to_string()
                    }
                    else
                    {
                        local_time.format("%Y-%m-%d %H:%M:%S").to_string()
                    };

                    let time_color = Color::from_rgb(0.40, 0.44, 0.50);
                    let event_color = Color::from_rgb(0.50, 0.54, 0.60);

                    let time_width = if local_time.date_naive() == today { 65.0 } else { 145.0 };
                    let time_label = text(timestamp).size(14).color(time_color)
                        .width(Length::Fixed(time_width));

                    let event_text = match kind
                    {
                        EventKind::Joined => format!("* {} has joined the room", nick),
                        EventKind::Left => format!("* {} has left the room", nick),
                        EventKind::StatusChanged(show) => match show.as_deref()
                        {
                            None => format!("* {} is now online", nick),
                            Some("away") => format!("* {} is now away", nick),
                            Some("xa") => format!("* {} is now extended away", nick),
                            Some("dnd") => format!("* {} is do not disturb", nick),
                            Some("chat") => format!("* {} is free for chat", nick),
                            Some(other) => format!("* {} status: {}", nick, other),
                        },
                    };

                    let event_label = text(event_text).size(14).color(event_color).width(Fill);

                    let event_row = row![time_label, event_label].spacing(4).width(Fill);
                    let event_element: Element<'_, Message> = container(event_row)
                        .padding(4)
                        .width(Fill)
                        .into();
                    messages.push(event_element);
                }
            }
        }

        let message_area = scrollable(
            column(messages).spacing(2).width(Fill)
        )
        .id(Id::new(MESSAGE_SCROLL_ID))
        .height(Fill)
        .width(Fill);

        // Input bar.
        let input = text_input("Type a message...", &state.message_input)
            .id(Id::new(MESSAGE_INPUT_ID))
            .on_input(Message::InputChanged)
            .on_submit(Message::SendMessage)
            .padding(10)
            .width(Fill);

        let send_btn = button(text("Send").size(14))
            .on_press(Message::SendMessage)
            .padding(10);

        let input_row = row![input, send_btn].spacing(8).width(Fill);

        return column![topic_label, message_area, input_row]
            .spacing(8)
            .width(Fill)
            .height(Fill)
            .padding(8)
            .into();
    }
    else
    {
        return join::view(state);
    }
}
