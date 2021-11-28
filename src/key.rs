pub enum Key {
    W,
    A,
    S,
    D,
    Q,
}

impl Key {
    pub const fn from_byte(key: u8) -> Option<Self> {
        match key {
            119 => Some(Self::W),
            97 => Some(Self::A),
            115 => Some(Self::S),
            100 => Some(Self::D),
            113 => Some(Self::Q),
            _ => None,
        }
    }
}
