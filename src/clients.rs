use crate::{Arc, RwLock, TcpStream, Write};

pub struct Clients(pub Vec<Option<Arc<RwLock<TcpStream>>>>);

impl Clients {
    pub fn set(&mut self, client: Arc<RwLock<TcpStream>>, client_i: usize) {
        self.0[client_i] = Some(client)
    }

    pub fn remove(&mut self, client_i: usize) {
        self.0[client_i] = None
    }

    pub fn find_vacancy(&self) -> Option<usize> {
        for i in 0..self.0.len() {
            if let None = self.0[i] {
                return Some(i);
            }
        }
        None
    }

    pub fn distribute(&mut self, area: &[u8], open: Arc<RwLock<bool>>) {
        //  Lock
        if let Ok(mut open_guard) = open.write() {
            *open_guard = false
        }

        let mut failed = Vec::<usize>::new();

        for i in 0..10 {
            if let Some(client) = &self.0[i] {
                if let Ok(mut client_guard) = client.write() {
                    if let Err(_) = client_guard.write_all(area) {
                        failed.push(i)
                    }
                }
            }
        }

        for i in failed.into_iter() {
            self.0[i] = None
        }
        //  Unlock
        if let Ok(mut open_guard) = open.write() {
            *open_guard = true
        }
    }
}
