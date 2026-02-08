use crate::events::key::Key;

pub enum Event {
    Input(Key),
    Tick,
}
