use {
    crate::{Direction, Direction::*},
    rand::{thread_rng, Rng},
};

pub struct Area {
    pub data: Vec<u8>,
    pub rows: usize,
    pub columns: usize,
}

impl Area {
    pub fn to_player(&mut self, area_i: usize) {
        self.data[area_i] = 111;
    }

    pub fn to_empty(&mut self, area_i: usize) {
        self.data[area_i] = 32
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

    pub fn attempt_move(&mut self, direction: Direction, area_i: usize) -> Option<usize> {
        let new = self.can_move(direction, area_i)?;

        self.to_empty(area_i);
        self.to_player(new);

        Some(new)
    }

    fn can_move(&self, direction: Direction, area_i: usize) -> Option<usize> {
        let mut p = area_i.clone();

        match direction {
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
        }

        if p != area_i && self.data[p] != 111 {
            Some(p)
        } else {
            None
        }
    }
}
