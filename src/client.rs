use std::{
    io::{self, Read, Write},
    net::TcpStream,
    str::from_utf8,
    thread,
};

use crate::consts::MSG_BUF_SIZE;

pub fn start() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:9999")?;
    let mut stream_clone = stream.try_clone()?;

    // 将接收和发送分开执行
    thread::spawn(move || {
        let mut buffer: Vec<u8> = vec![0; MSG_BUF_SIZE];
        loop {
            if let Ok(msg_size) = stream_clone.read(&mut buffer) {
                if msg_size > 0 {
                    println!(
                        "Server broadcast: {}",
                        from_utf8(&buffer[..msg_size]).unwrap()
                    )
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
        stream.write(msg_bytes).expect("Failed to write!");
    }
}
