use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    str::from_utf8,
    sync::mpsc::{self, Sender},
    thread,
};

use crate::consts::MSG_BUF_SIZE;

fn handle_client(mut stream: TcpStream, sender: Sender<String>) -> std::io::Result<()> {
    // let mut buffer: Vec<u8> = vec![]; 为什么不可以……因为没有指定缓冲大小吗……
    let mut buffer: Vec<u8> = vec![0; MSG_BUF_SIZE];
    let client_addr = stream.peer_addr().expect("Failed to get client addr.");
    println!("Client {} has been online.", client_addr);
    loop {
        if let Ok(msg_size) = stream.read(&mut buffer) {
            if msg_size > 0 {
                let msg = from_utf8(&buffer[..msg_size]).unwrap();
                println!("Client {}: {}", client_addr, msg);
                sender.send(msg.to_string()).expect("Failed to send msg.");
                println!("Sent to receiver")
            }
        } else {
            println!("Client {} is offline now.", client_addr);
            break;
        }
    }
    return Ok(());
}

pub fn start() -> std::io::Result<()> {
    // let mut handler_threads: Vec<thread::JoinHandle<()>> = vec![];
    let mut clients: Vec<TcpStream> = vec![];

    let (msg_sender, msg_receiver) = mpsc::channel::<String>();
    let (stream_sender, stream_receiver) = mpsc::channel::<TcpStream>();

    thread::spawn(move || {
        let listener = TcpListener::bind("127.0.0.1:9999").expect("Failed to bind.");
        for stream in listener.incoming() {
            let stream = stream.expect("Failed to get stream.");
            let msg_sender_clone = msg_sender.clone();
            stream_sender
                .send(stream.try_clone().expect("Failed to clone stream."))
                .expect("Failed to send stream.");
            thread::spawn(move || {
                handle_client(stream, msg_sender_clone).unwrap_or_else(|err| eprintln!("{:?}", err))
            });
        }
    });

    loop {
        if let Ok(stream) = stream_receiver.try_recv() {
            println!("Stream pushed.");
            clients.push(stream);
        }

        if let Ok(msg) = msg_receiver.try_recv() {
            println!("Msg received, try to broadcast...");
            for mut client in &clients {
                client
                    .write(msg.as_bytes())
                    .expect("Failed to send msg to client");
            }
        }
    }
}
