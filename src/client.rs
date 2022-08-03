use std::{
    io::{self, BufRead, BufReader, Write},
    net::TcpStream,
    str::from_utf8,
};

pub fn init() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:9999")?;

    for _ in 0..10 {
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input!");
        stream.write(input.as_bytes()).expect("Failed to write!");

        let mut reader = BufReader::new(&stream);
        let mut buffer: Vec<u8> = vec![];
        reader
            .read_until(b'\n', &mut buffer)
            .expect("Failed to read from buffer!");
        println!("S to C: {}", from_utf8(&buffer).unwrap());
    }
    return Ok(());
}
