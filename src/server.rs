use std::{
    collections::HashMap,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    str::from_utf8,
    sync::mpsc::{self, Sender},
    thread,
};

use crate::{
    consts::MSG_BUF_SIZE,
    message::{Message, MessageType},
};

/// TcpStream with a temp id
struct WrapedStream {
    stream_id: u32,
    w_stream: TcpStream,
}
impl Clone for WrapedStream {
    fn clone(&self) -> Self {
        Self {
            stream_id: self.stream_id.clone(),
            w_stream: self.w_stream.try_clone().expect("Failed to clone stream."),
        }
    }
}

fn handle_client(mut client: WrapedStream, sender: Sender<Message>) -> std::io::Result<()> {
    // let mut buffer: Vec<u8> = vec![]; 为什么不可以……因为没有指定缓冲大小吗……
    let mut buffer: Vec<u8> = vec![0; MSG_BUF_SIZE];
    let client_addr = client
        .w_stream
        .peer_addr()
        .expect("Failed to get client addr.");
    println!("Client {} has been online.", client_addr);
    loop {
        if let Ok(msg_size) = client.w_stream.read(&mut buffer) {
            if msg_size > 0 {
                // read msg string and convert it to type Message
                let msg_str = from_utf8(&buffer[..msg_size]).unwrap();
                let msg = Message::convert_to_msg(msg_str);
                println!("Client {}: {}", client_addr, msg);
                sender.send(msg).expect("Failed to send msg.");
                println!("Sent to receiver")
            }
        } else {
            // client has left, should delete its stream
            let exit_message = Message {
                msg_type: MessageType::ClientExit,
                sender_name: client.stream_id.to_string(),
                msg_content: format!(
                    "Client {} with id {} is offline now.",
                    client_addr, client.stream_id
                ),
            };
            sender.send(exit_message).expect("Failed to send exit msg.");
            break;
        }
    }

    return Ok(());
}

pub fn start() -> std::io::Result<()> {
    banner_na::banner("CHAMBER").expect("Failed to render banner.");
    banner_na::banner("SERVER").expect("Failed to render banner.");

    let mut clients: HashMap<u32, TcpStream> = HashMap::default();

    let (msg_sender, msg_receiver) = mpsc::channel::<Message>();
    let (client_sender, client_receiver) = mpsc::channel::<WrapedStream>();

    // a thread to get connections
    thread::spawn(move || {
        let listener = TcpListener::bind("127.0.0.1:9999").expect("Failed to bind.");
        let mut id: u32 = 0;
        for stream in listener.incoming() {
            let stream = stream.expect("Failed to get stream.");
            let client = WrapedStream {
                stream_id: id,
                w_stream: stream,
            };
            let msg_sender_clone = msg_sender.clone();
            client_sender
                .send(client.clone())
                .expect("Failed to send client.");
            // create a new thread to handle a connection
            thread::spawn(move || {
                handle_client(client, msg_sender_clone).unwrap_or_else(|err| eprintln!("{:?}", err))
            });
            // may overflow, ha ha
            id += 1;
        }
    });

    loop {
        if let Ok(client) = client_receiver.try_recv() {
            println!("Stream pushed.");
            clients.insert(client.stream_id, client.w_stream);
        }

        if let Ok(msg) = msg_receiver.try_recv() {
            println!("Msg received, handle it...");
            // if client has been offline, server will panic
            match msg.msg_type {
                crate::message::MessageType::ClientLogIn => {}
                crate::message::MessageType::ClientExit => {
                    clients.remove(
                        &msg.sender_name
                            .parse::<u32>()
                            .expect("Failed to get id to remove."),
                    );
                    println!("{}", msg);
                }
                crate::message::MessageType::ClientListUpdate => {}
                crate::message::MessageType::TextMessage => {
                    for (_, mut client) in &clients {
                        client
                            .write(msg.to_string().as_bytes())
                            .expect("Failed to send msg to client");
                    }
                }
                crate::message::MessageType::Error => {}
            }
        }
    }
}
