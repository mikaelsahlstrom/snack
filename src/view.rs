use iced::{ Element, Fill };
use iced::widget::row;

use crate::app::{ AppState, Selection, Snack };
use crate::message::Message;
use crate::ui;

impl Snack
{
    pub(crate) fn view(&self) -> Element<'_, Message>
    {
        match self.state
        {
            AppState::Login | AppState::Connecting =>
            {
                return ui::account::view(self);
            }
            AppState::Connected =>
            {
                let room_list = ui::sidebar::view(self);
                let center = ui::chat::view(self);

                let show_members = matches!(self.active, Some(Selection::Room(_)));

                if show_members
                {
                    let member_list = ui::members::view(self);
                    return row![room_list, center, member_list]
                        .spacing(0)
                        .height(Fill)
                        .width(Fill)
                        .into();
                }

                return row![room_list, center]
                    .spacing(0)
                    .height(Fill)
                    .width(Fill)
                    .into();
            }
        }
    }
}
