mod pool;
use anyhow::{Context, Result};
use std::{
    fs,
    io::{BufRead, BufReader, BufWriter, Write},
    net::{TcpListener, TcpStream},
};
fn main() -> Result<()> {
    let listener =
        TcpListener::bind("127.0.0.1:8080").context("Unable to bind at given address")?;

    // Keep on listening for new requests
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => handle_stream(stream).context("Issue with handle stream")?,
            Err(e) => println!("Can't accept stream because of {e}"),
        }
    }
    Ok(())
}

fn handle_stream(stream: TcpStream) -> Result<()> {
    let reader = BufReader::new(&stream);
    let request = reader.lines().next().unwrap().unwrap();

    let (response_header, file_path) = match &request[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "./response/home.html"),
        "GET /test HTTP/1.1" => ("HTTP/1.1 200 OK", "./response/test.html"),
        _ => ("HTTP/1.1 404 NotFound", "./response/not_found.html"),
    };

    let response = fs::read_to_string(file_path).expect("Unable to read file");
    let length = response.len();
    let mut writer = BufWriter::new(stream);
    writer
        .write_all(format!("{response_header}\nContent-Length: {length}\n{response}\n").as_bytes())
        .expect("Unable to write to stream");

    Ok(())
}
