mod area;
mod clients;
mod direction;

use {
    area::Area,
    clients::Clients,
    direction::Direction,
    std::{
        env::args,
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

fn handle_client(
    area_i: Arc<RwLock<usize>>,
    client: Arc<RwLock<TcpStream>>,
    area: Arc<RwLock<Area>>,
    clients: Arc<RwLock<Clients>>,
    open: Arc<RwLock<bool>>,
) -> Result<()> {
    Ok(if let Ok(area_guard) = area.read() {
        if let Ok(mut client_guard) = client.write() {
            client_guard.write_all(&[*area_i.read().unwrap() as u8])?;

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
                                            if let Some(direction) = Direction::from_key(key[0]) {
                                                if let Some(i) = area_guard
                                                    .attempt_move(direction, *area_i_guard)
                                                {
                                                    drop(area_guard);
                                                    drop(area_i_guard);

                                                    if let Ok(mut area_i_guard) = area_i.write() {
                                                        *area_i_guard = i;
                                                        drop(area_i_guard);

                                                        if let Ok(mut clients_guard) =
                                                            clients.write()
                                                        {
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
                                }
                                Err(e) => match e.kind() {
                                    TimedOut => sleep(WAIT),
                                    ConnectionReset => return Err(e),
                                    _ => unreachable!(),
                                },
                            }
                        }
                    }
                }
            }
        }
    })
}

fn main() {
    let area = Arc::new(RwLock::new({
        let (rows, columns) = (10, 10);
        Area {
            data: (0..columns * rows).map(|_| 32).collect(),
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

    if let Ok(listener) = TcpListener::bind(
        if let Some(tsa) = (|| -> Option<String> {
            let mut a = args();
            if a.len() == 3 {
                let mut tsa = "".to_owned();
                tsa.push_str(&a.nth(1)?);
                tsa.push(':');
                tsa.push_str(&a.next()?);
                Some(tsa)
            } else {
                None
            }
        })() {
            tsa
        } else {
            String::from("127.0.0.1:6969")
        },
    ) {
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

                                                if let Err(e) = handle_client(
                                                    area_i.clone(),
                                                    client,
                                                    thread_area.clone(),
                                                    thread_clients.clone(),
                                                    thread_open.clone(),
                                                ) {
                                                    println!("{:?}", e)
                                                }

                                                if let Ok(mut clients_guard) =
                                                    thread_clients.write()
                                                {
                                                    clients_guard.remove(client_i)
                                                }
                                                if let Ok(mut area_guard) = thread_area.write() {
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
            });
        }
    }
}
