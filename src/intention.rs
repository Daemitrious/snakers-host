use crate::{Key, Key::*};

pub enum Intention {
    Move(Key),
    Exit,
}

impl From<Key> for Intention {
    fn from(key: Key) -> Self {
        match key {
            W | A | S | D => Self::Move(key),
            Q => Self::Exit,
        }
    }
}
