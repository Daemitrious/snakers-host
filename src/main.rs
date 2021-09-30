use std::{
    io::{Error, Read, Write},
    net::{TcpListener, TcpStream},
    sync::{mpsc::channel, Arc, Mutex},
    thread::{spawn, Builder, JoinHandle},
};

fn main() -> Result<(), Error> {
    //  Idea | Annotate an enumeration pertaining to every possible practical distribution
    let (sender, receiver) = channel::<u8>();
    let open = Arc::new(Mutex::new(true));

    let stream_pool = Arc::new(Mutex::new(Vec::<Arc<Mutex<TcpStream>>>::new()));
    let mut thread_pool = Vec::<JoinHandle<()>>::new();

    let listener_stream_pool_thread = stream_pool.clone();
    let listener_thread_open = open.clone();

    //  Listener thread
    Builder::new()
        .name("tcplistener".to_string())
        .spawn(move || -> Result<(), Error> {
            for stream in TcpListener::bind("127.0.0.1:6969")?.incoming() {
                let thread_sender = sender.clone();
                let thread_stream_pool = listener_stream_pool_thread.clone();

                let thread_open = listener_thread_open.clone();

                //  Client thread
                thread_pool.push(spawn(move || {
                    if let Ok(stream) = stream {
                        if let Ok(mut thread_stream_pool_lock) = thread_stream_pool.lock() {
                            thread_stream_pool_lock.push(Arc::new(Mutex::new(stream)));
                            drop(thread_stream_pool_lock);

                            loop {
                                if *thread_open.lock().unwrap() {
                                    if let Ok(thread_stream_pool_lock) = thread_stream_pool.lock() {
                                        let buf = &mut [0; 1];
                                        thread_stream_pool_lock[thread_stream_pool_lock.len() - 1]
                                            .lock()
                                            .unwrap()
                                            .read_exact(buf)
                                            .unwrap();
                                        //  Unlock | ASAP
                                        drop(thread_stream_pool_lock);

                                        if let Some(v) = buf.first() {
                                            thread_sender.send(*v).unwrap();
                                        }
                                        *thread_open.lock().unwrap() = false;
                                    }
                                }
                            }
                        }
                    } else {
                        println!("Stream failed to connect")
                    }
                }))
            }
            Ok(())
        })?;

    loop {
        if let Ok(msg) = receiver.recv() {
            match msg {
                //  Tend to other possibilites
                _ => {
                    if let Ok(stream_pool_lock) = stream_pool.lock() {
                        for i in 0..stream_pool_lock.len() {
                            if let Ok(mut lock) = stream_pool_lock[i].lock() {
                                lock.write(&[0; 1]).unwrap();
                            }
                        }
                        *open.lock().unwrap() = true;
                    }
                }
            }
        }
    }
}
