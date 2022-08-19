use std::{
    io::{self, Read, Write},
    net::TcpStream,
    str::from_utf8,
    thread,
};

use crate::{
    consts::MSG_BUF_SIZE,
    message::{Message, MessageType},
};

pub fn start() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:9999")?;
    let mut stream_clone = stream.try_clone()?;

    // create a new thread to receive msg from server
    thread::spawn(move || {
        let mut buffer: Vec<u8> = vec![0; MSG_BUF_SIZE];
        loop {
            if let Ok(msg_size) = stream_clone.read(&mut buffer) {
                if msg_size > 0 {
                    let msg = Message::convert_to_msg(from_utf8(&buffer[..msg_size]).unwrap());
                    println!("Server broadcast: {}", msg)
                }
            } else {
                println!("Server is offline now.");
                break;
            }
        }
    });

    loop {
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input!");
        let msg_bytes = input.as_bytes();
        println!("Msg size: {} bytes.", msg_bytes.len());
        let msg = Message {
            msg_type: MessageType::TextMessage,
            sender_name: "local client: ".to_owned() + &stream.local_addr().unwrap().to_string(),
            msg_content: from_utf8(msg_bytes).unwrap_or_default().to_string(),
        };
        stream
            .write(msg.to_string().as_bytes())
            .expect("Failed to write!");
    }
}
