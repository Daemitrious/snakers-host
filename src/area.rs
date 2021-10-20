use rand::{thread_rng, Rng};

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

    pub fn move_player(&mut self, key: u8, area_i: usize) -> Option<usize> {
        let mut p = area_i.clone();

        if match key {
            119 => {
                if area_i / self.rows > 0 {
                    p -= self.rows;
                    self.data[p] == 32
                } else {
                    false
                }
            }
            97 => {
                if area_i % self.columns > 0 {
                    p -= 1;
                    self.data[p] == 32
                } else {
                    false
                }
            }
            115 => {
                if area_i / self.rows < self.rows - 1 {
                    p += self.rows;
                    self.data[p] == 32
                } else {
                    false
                }
            }
            100 => {
                if area_i % self.columns < self.columns - 1 {
                    p += 1;
                    self.data[p] == 32
                } else {
                    false
                }
            }
            _ => unreachable!(),
        } {
            self.to_empty(area_i);
            self.to_player(p);
            Some(p)
        } else {
            None
        }
    }
}
