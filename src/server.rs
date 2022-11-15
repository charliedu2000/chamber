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
struct WrappedStream {
    stream_id: u32,
    stream: TcpStream,
}
impl Clone for WrappedStream {
    fn clone(&self) -> Self {
        Self {
            stream_id: self.stream_id.clone(),
            stream: self.stream.try_clone().expect("Failed to clone stream."),
        }
    }
}

/// receiver 只能被一个线程所拥有，stream 接收到消息之后，广播交给 server 来进行……
///
/// 但是在 server 线程中更新客户端列表时会不会略繁琐……
fn handle_client(mut client: WrappedStream, sender: Sender<Message>) -> std::io::Result<()> {
    // let mut buffer: Vec<u8> = vec![]; 为什么不可以……因为没有指定缓冲大小吗……
    let mut buffer: Vec<u8> = vec![0; MSG_BUF_SIZE];
    let client_addr = client
        .stream
        .peer_addr()
        .expect("Failed to get client addr.");
    println!("Client {} has been online.", client_addr);
    loop {
        if let Ok(msg_size) = client.stream.read(&mut buffer) {
            if msg_size > 0 {
                // read msg string and convert it to type Message
                let msg_str = from_utf8(&buffer[..msg_size]).unwrap();
                let msg = Message::convert_to_msg(msg_str);
                println!("Client {}: {}", client_addr, msg);
                sender.send(msg).expect("Failed to send msg.");
                println!("Sent to receiver")
            }
        } else {
            // client has been offline, delete its stream
            let exit_message = Message {
                msg_type: MessageType::ClientExit,
                msg_sender: client.stream_id.to_string(),
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
    let slant_font = figlet_rs::FIGfont::standard().unwrap();
    let figure = slant_font.convert("Chamber");
    assert!(figure.is_some());
    println!("{}", figure.unwrap());

    let mut clients: HashMap<u32, TcpStream> = HashMap::default();

    let (msg_sender, msg_receiver) = mpsc::channel::<Message>();
    let (client_sender, client_receiver) = mpsc::channel::<WrappedStream>();

    // a thread to get connections
    thread::spawn(move || {
        let listener = TcpListener::bind("127.0.0.1:9999").expect("Failed to bind.");
        let mut id: u32 = 0;
        for new_stream in listener.incoming() {
            let new_stream = new_stream.expect("Failed to get stream.");
            let client = WrappedStream {
                stream_id: id,
                stream: new_stream,
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
            clients.insert(client.stream_id, client.stream);
        }

        if let Ok(msg) = msg_receiver.try_recv() {
            println!("Msg received, handle it...");
            // handle msg
            match msg.msg_type {
                crate::message::MessageType::ClientLogIn => {
                    // send updated client list to all clients
                }
                crate::message::MessageType::ClientExit => {
                    let index = msg
                        .msg_sender
                        .parse::<u32>()
                        .expect("Failed to get index of exited client.");
                    clients
                        .get(&index)
                        .expect("Failed to get exited client.")
                        .shutdown(std::net::Shutdown::Both)?;
                    clients.remove(&index);
                    println!("{}", msg);
                }
                crate::message::MessageType::TextMessage => {
                    // send msg to all clients
                    for (_, mut client) in &clients {
                        client
                            .write(msg.to_string().as_bytes())
                            .expect("Failed to send msg to client");
                    }
                }
                crate::message::MessageType::Error => {}
                _ => {}
            }
        }
    }
}
