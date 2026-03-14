use iced::{ Element, Fill };
use iced::widget::{ button, column, container, row, scrollable, text, text_input, Id };

use crate::{ Message, Snack, MESSAGE_SCROLL_ID, MESSAGE_INPUT_ID };
use crate::ui::join;

pub fn view(state: &Snack) -> Element<'_, Message>
{
    if state.show_join_panel
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
        let messages: Vec<Element<'_, Message>> = room.messages.iter().map(|m|
        {
            let local_time = m.received.with_timezone(&chrono::Local);
            let timestamp = if local_time.date_naive() == today
            {
                local_time.format("%H:%M:%S").to_string()
            }
            else
            {
                local_time.format("%Y-%m-%d %H:%M:%S").to_string()
            };
            let line = text(format!("[{}] {}: {}", timestamp, m.from, m.body)).size(14);

            container(line).padding(4).width(Fill).into()
        }).collect();

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

        column![topic_label, message_area, input_row]
            .spacing(8)
            .width(Fill)
            .height(Fill)
            .padding(8)
            .into()
    }
    else
    {
        container(text("Select a room").size(16))
            .center(Fill)
            .width(Fill)
            .height(Fill)
            .into()
    }
}
