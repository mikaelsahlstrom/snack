#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use iced::{ Application, Program, Theme };

mod app;
mod message;
mod room;
mod storage;
mod subscription;
mod ui;
mod update;
mod view;
mod xmpp;

pub use app::{
    AppState, NickCompleteState, Selection, Snack,
    ACCOUNT_JID_INPUT_ID, ACCOUNT_PASSWORD_INPUT_ID, JOIN_INPUT_ID,
    MESSAGE_INPUT_ID, MESSAGE_SCROLL_ID,
};
pub use message::Message;

fn main() -> iced::Result
{
    env_logger::init();
    return application().run();
}

fn application() -> Application<impl Program<Message = Message, Theme = Theme>>
{
    return iced::application(Snack::new, Snack::update, Snack::view)
                .subscription(Snack::subscription)
                .title(Snack::title)
                .theme(Snack::theme);
}
