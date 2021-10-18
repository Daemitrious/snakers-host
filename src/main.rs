use {
    rand::{thread_rng, Rng},
    std::{
        io::{
            ErrorKind::{ConnectionReset, TimedOut},
            Read, Result, Write,
        },
        net::{TcpListener, TcpStream},
        sync::{Arc, RwLock},
        thread::{sleep, spawn},
        time::Duration,
    },
};

const W: u8 = 119;
const A: u8 = 97;
const S: u8 = 115;
const D: u8 = 100;

const EMPTY: u8 = 32;
const PLAYER: u8 = 111;

struct Area {
    data: Vec<u8>,
    rows: usize,
    columns: usize,
}

impl Area {
    fn to_player(&mut self, area_i: usize) {
        self.data[area_i] = PLAYER;
    }

    fn to_empty(&mut self, area_i: usize) {
        self.data[area_i] = EMPTY
    }

    fn find_vacancy(&self) -> Option<usize> {
        let mut vacancies = Vec::new();

        for i in 0..self.data.len() {
            if self.data[i] == EMPTY {
                vacancies.push(i)
            }
        }
        match vacancies.len() {
            0 => None,
            1 => Some(vacancies[0]),
            l @ _ => Some(vacancies[thread_rng().gen_range(0..l)]),
        }
    }

    fn move_player(&mut self, key: u8, area_i: usize) -> Option<usize> {
        let mut p = area_i.clone();

        if match key {
            W => {
                if area_i / self.rows > 0 {
                    p -= self.rows;
                    self.data[p] == EMPTY
                } else {
                    false
                }
            }
            A => {
                if area_i % self.columns > 0 {
                    p -= 1;
                    self.data[p] == EMPTY
                } else {
                    false
                }
            }
            S => {
                if area_i / self.rows < self.rows - 1 {
                    p += self.rows;
                    self.data[p] == EMPTY
                } else {
                    false
                }
            }
            D => {
                if area_i % self.columns < self.columns - 1 {
                    p += 1;
                    self.data[p] == EMPTY
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

struct Clients(Vec<Option<Arc<RwLock<TcpStream>>>>);

impl Clients {
    fn set(&mut self, client: Arc<RwLock<TcpStream>>, client_i: usize) {
        self.0[client_i] = Some(client)
    }

    fn remove(&mut self, client_i: usize) {
        self.0[client_i] = None
    }

    fn find_vacancy(&self) -> Option<usize> {
        for i in 0..self.0.len() {
            if let None = self.0[i] {
                return Some(i);
            }
        }
        None
    }

    fn distribute(&mut self, area: &[u8], open: Arc<RwLock<bool>>) {
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

fn handle_client(
    area_i: Arc<RwLock<usize>>,
    client: Arc<RwLock<TcpStream>>,
    area: Arc<RwLock<Area>>,
    clients: Arc<RwLock<Clients>>,
    open: Arc<RwLock<bool>>,
) -> Result<()> {
    if let Ok(area_guard) = area.read() {
        if let Ok(mut client_guard) = client.write() {
            client_guard.write_all(&[area_guard.rows as u8])?;
            client_guard.write_all(&[area_guard.columns as u8])?;
            client_guard.write_all(&area_guard.data)?;

            const WAIT: Duration = Duration::from_micros(1);

            client_guard.set_read_timeout(Some(WAIT))?;

            drop(area_guard);
            drop(client_guard);

            loop {
                if let Ok(open_guard) = open.read() {
                    if *open_guard {
                        drop(open_guard);

                        if let Ok(mut client_guard) = client.write() {
                            let key = &mut [0; 1];

                            match client_guard.read_exact(key) {
                                Ok(()) => {
                                    drop(client_guard);

                                    if let Ok(mut area_guard) = area.write() {
                                        if let Ok(area_i_guard) = area_i.read() {
                                            if let Some(i) =
                                                area_guard.move_player(key[0], *area_i_guard)
                                            {
                                                drop(area_guard);
                                                drop(area_i_guard);

                                                if let Ok(mut area_i_guard) = area_i.write() {
                                                    *area_i_guard = i;
                                                    drop(area_i_guard);

                                                    if let Ok(mut clients_guard) = clients.write() {
                                                        if let Ok(area_guard) = area.read() {
                                                            clients_guard.distribute(
                                                                &area_guard.data,
                                                                open.clone(),
                                                            )
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                Err(e) => match e.kind() {
                                    TimedOut => sleep(WAIT), //  Temporary CPU Usage fix
                                    ConnectionReset => return Err(e),
                                    _ => unreachable!(),
                                },
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn main() {
    let area = Arc::new(RwLock::new({
        let (rows, columns) = (5, 5);
        Area {
            data: (0..columns * rows).map(|_| EMPTY).collect(),
            rows,
            columns,
        }
    }));

    let clients = Arc::new(RwLock::new(Clients(
        (0..10)
            .map(|_| None)
            .collect::<Vec<Option<Arc<RwLock<TcpStream>>>>>(),
    )));

    let open = Arc::new(RwLock::new(true));

    let thread_area = area.clone();
    let thread_clients = clients.clone();
    let thread_open = open.clone();

    if let Ok(listener) = TcpListener::bind("127.0.0.1:6969") {
        for stream in listener.incoming() {
            let thread_area = thread_area.clone();
            let thread_clients = thread_clients.clone();
            let thread_open = thread_open.clone();

            spawn(move || {
                if let Ok(client) = stream {
                    if let Ok(clients_guard) = thread_clients.read() {
                        if let Some(client_i) = clients_guard.find_vacancy() {
                            drop(clients_guard);

                            if let Ok(area_guard) = thread_area.read() {
                                if let Some(area_i) = area_guard.find_vacancy() {
                                    drop(area_guard);

                                    if let Ok(mut area_guard) = thread_area.write() {
                                        area_guard.to_player(area_i);
                                        drop(area_guard);

                                        if let Ok(mut clients_guard) = thread_clients.write() {
                                            if let Ok(area_guard) = thread_area.read() {
                                                clients_guard.distribute(
                                                    &area_guard.data,
                                                    thread_open.clone(),
                                                );
                                                drop(area_guard);

                                                let client = Arc::new(RwLock::new(client));
                                                clients_guard.set(client.clone(), client_i);
                                                drop(clients_guard);

                                                let area_i = Arc::new(RwLock::new(area_i));

                                                if let Err(_) = handle_client(
                                                    area_i.clone(),
                                                    client,
                                                    thread_area.clone(),
                                                    thread_clients.clone(),
                                                    thread_open.clone(),
                                                ) {
                                                    if let Ok(mut clients_guard) =
                                                        thread_clients.write()
                                                    {
                                                        clients_guard.remove(client_i)
                                                    }
                                                    if let Ok(mut area_guard) = thread_area.write()
                                                    {
                                                        if let Ok(area_i_guard) = area_i.read() {
                                                            area_guard.to_empty(*area_i_guard);
                                                            drop(area_guard);

                                                            if let Ok(mut clients_guard) =
                                                                thread_clients.write()
                                                            {
                                                                if let Ok(area_guard) =
                                                                    thread_area.read()
                                                                {
                                                                    clients_guard.distribute(
                                                                        &area_guard.data,
                                                                        thread_open.clone(),
                                                                    )
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            });
        }
    }
}
