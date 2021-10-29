pub enum Direction {
    W,
    A,
    S,
    D,
}

impl Direction {
    pub fn from_key(key: u8) -> Option<Self> {
        match key {
            119 => Some(Self::W),
            97 => Some(Self::A),
            115 => Some(Self::S),
            100 => Some(Self::D),
            _ => None,
        }
    }
}
