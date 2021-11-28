mod area;
mod client;
mod intention;
mod key;

use {
    area::Area,
    client::Clients,
    intention::{Intention, Intention::*},
    key::Key,
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

type Lock<T> = Arc<RwLock<T>>;

fn lock<T>(element: T) -> Lock<T> {
    Arc::new(RwLock::new(element))
}

fn handle_client(
    position: Lock<usize>,
    client: Lock<TcpStream>,
    area: Lock<Area>,
    clients: Lock<Clients>,
    open: Lock<bool>,
) -> Result<()> {
    Ok(if let Ok(area_guard) = area.read() {
        if let Ok(mut client_guard) = client.write() {
            client_guard.write_all(&[*position.read().unwrap() as u8])?;

            client_guard.write_all(&area_guard.get_rows_byte())?;
            client_guard.write_all(&area_guard.get_columns_byte())?;

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

                            let read_result = client_guard.read_exact(key);
                            drop(client_guard);

                            if let Err(error) = read_result {
                                match error.kind() {
                                    TimedOut => sleep(WAIT),
                                    ConnectionReset => return Err(error),
                                    _ => {
                                        println!("Unreachable:");
                                        return Err(error);
                                    }
                                }
                            } else {
                                if let Some(key) = Key::from_byte(key[0]) {
                                    match Intention::from(key) {
                                        Move(direction) => {
                                            if let Ok(mut area_guard) = area.write() {
                                                if let Ok(position_guard) = position.read() {
                                                    if let Some(i) = area_guard
                                                        .attempt_move(direction, *position_guard)
                                                    {
                                                        drop(area_guard);
                                                        drop(position_guard);

                                                        if let Ok(mut client_guard) = client.write() {
                                                            client_guard.write_all(&[0; 1])?;
                                                            drop(client_guard);
    
                                                            if let Ok(mut position_guard) =
                                                                position.write()
                                                            {
                                                                *position_guard = i;
                                                                drop(position_guard);
    
                                                                if let Ok(mut clients_guard) =
                                                                    clients.write()
                                                                {
                                                                    if let Ok(area_guard) = area.read()
                                                                    {
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
                                        Exit => {
                                            if let Ok(mut client_guard) = client.write() {
                                                client_guard.write_all(&[0])?;
                                            }
                                            //  Placed outside of previous statement in case of `Err(_)`
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    })
}

fn main() {
    let area = lock({
        let (rows, columns) = (10, 10);
        Area {
            data: (0..columns * rows).map(|_| 32).collect(),
            rows,
            columns,
        }
    });

    let clients = lock(Clients(
        (0..10)
            .map(|_| None)
            .collect::<Vec<Option<Lock<TcpStream>>>>(),
    ));

    let open = lock(true);

    let thread_area = area.clone();
    let thread_clients = clients.clone();
    let thread_open = open.clone();

    if let Ok(listener) = TcpListener::bind(
        if let Some(tsa) = (|| -> Option<String> {
            let mut arguments = args();
            if arguments.len() == 3 {
                let mut tsa = "".to_owned();
                tsa.push_str(&arguments.nth(1)?);
                tsa.push(':');
                tsa.push_str(&arguments.next()?);
                Some(tsa)
            } else {
                None
            }
        })() {
            tsa
        } else {
            String::from("127.0.0.1:8080")
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
                                if let Some(position) = area_guard.find_vacancy() {
                                    drop(area_guard);

                                    if let Ok(mut area_guard) = thread_area.write() {
                                        area_guard.to_player(position);
                                        drop(area_guard);

                                        if let Ok(mut clients_guard) = thread_clients.write() {
                                            if let Ok(area_guard) = thread_area.read() {
                                                clients_guard.distribute(
                                                    &area_guard.data,
                                                    thread_open.clone(),
                                                );
                                                drop(area_guard);

                                                let client = lock(client);
                                                clients_guard.set(client.clone(), client_i);
                                                drop(clients_guard);

                                                let position = lock(position);

                                                if let Err(error) = handle_client(
                                                    position.clone(),
                                                    client.clone(),
                                                    thread_area.clone(),
                                                    thread_clients.clone(),
                                                    thread_open.clone(),
                                                ) {
                                                    println!("{:?}", error)
                                                }

                                                if let Ok(mut clients_guard) =
                                                    thread_clients.write()
                                                {
                                                    clients_guard.remove(client_i)
                                                }
                                                if let Ok(mut area_guard) = thread_area.write() {
                                                    if let Ok(position_guard) = position.read() {
                                                        area_guard.to_empty(*position_guard);
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
