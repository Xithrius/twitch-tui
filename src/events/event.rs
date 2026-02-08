use crate::{
    events::key::Key,
    handlers::{data::RawMessageData, state::State},
};

pub enum Event {
    /// An event that is emitted on a regular schedule.
    ///
    /// Use this event to run any code which has to run outside of being a direct response to a user
    /// event. e.g. polling exernal systems, updating animations, or rendering the UI based on a
    /// fixed frame rate.
    Tick,
    /// Crossterm events translated to internal key type events
    Input(Key),
    /// Events cause by user interactions
    Internal(InternalEvent),
    /// Either events to be sent to the websocket handler or notifications that come from said handler
    Twitch(TwitchEvent),
}

pub enum InternalEvent {
    Quit,
    BackOneLayer,
    SwitchState(State),
    Enter(TwitchAction),
    OpenStream(String),
}

#[derive(Debug, Clone)]
pub enum TwitchAction {
    Message(String),
    JoinChannel(String),
}

pub enum TwitchNotification {
    Message(RawMessageData),
    ClearChat(Option<String>),
    DeleteMessage(String),
}

pub enum TwitchEvent {
    Action(TwitchAction),
    Notification(TwitchNotification),
}
