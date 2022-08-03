use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    str::from_utf8,
    thread,
};

fn handle_client(mut stream: TcpStream) -> std::io::Result<()> {
    let mut buffer = [0; 512];
    // let mut buffer: Vec<u8> = vec![]; 为什么不可以…… ？
    loop {
        let bytes_read = stream.read(&mut buffer)?;
        if bytes_read == 0 {
            return Ok(());
        }
        // 将接收到的内容发送回去
        println!("Client: {}", from_utf8(&buffer[..bytes_read]).unwrap());
        stream.write(&buffer[..bytes_read])?;
    }
}

pub fn init() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:9999")?;
    let mut threads: Vec<thread::JoinHandle<()>> = vec![];

    for stream in listener.incoming() {
        let stream = stream.expect("Failed!");
        // 创建线程处理流
        let handle = thread::spawn(move || {
            handle_client(stream).unwrap_or_else(|error| eprintln!("{:?}", error));
        });

        threads.push(handle);
    }

    for handle in threads {
        handle.join().unwrap();
    }
    Ok(())
}
