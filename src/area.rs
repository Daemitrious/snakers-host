use {
    crate::{Key, Key::*},
    rand::{thread_rng, Rng},
};

pub struct Area {
    pub data: Vec<u8>,
    pub rows: usize,
    pub columns: usize,
}

impl Area {
    pub const fn get_rows_byte(&self) -> [u8; 1] {
        [self.rows as u8]
    }

    pub const fn get_columns_byte(&self) -> [u8; 1] {
        [self.columns as u8]
    }

    pub fn to_player(&mut self, position: usize) {
        self.data[position] = 111;
    }

    pub fn to_empty(&mut self, position: usize) {
        self.data[position] = 32
    }

    pub fn find_vacancy(&self) -> Option<usize> {
        let mut vacancies = Vec::new();

        for i in 0..self.data.len() {
            if self.data[i] == 32 {
                vacancies.push(i)
            }
        }
        match vacancies.len() {
            0 => None,
            1 => Some(vacancies[0]),
            l @ _ => Some(vacancies[thread_rng().gen_range(0..l)]),
        }
    }

    pub fn attempt_move(&mut self, key: Key, position: usize) -> Option<usize> {
        let new = self.can_move(key, position)?;

        self.to_empty(position);
        self.to_player(new);

        Some(new)
    }

    fn can_move(&self, key: Key, position: usize) -> Option<usize> {
        let mut p = position.clone();

        match key {
            W => {
                if p / self.rows > 0 {
                    p -= self.rows;
                }
            }
            A => {
                if p % self.columns > 0 {
                    p -= 1;
                }
            }
            S => {
                if p / self.rows < self.rows - 1 {
                    p += self.rows;
                }
            }
            D => {
                if p % self.columns < self.columns - 1 {
                    p += 1;
                }
            }
            _ => unreachable!(),
        }

        if p != position && self.data[p] != 111 {
            Some(p)
        } else {
            None
        }
    }
}
