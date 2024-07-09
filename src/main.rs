use std::{fs, io, thread};
use std::error::Error;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;

const PORT: &str = "7878";

fn main() {
    run().unwrap_or_else(|err| {
        println!("Finished with error: {err}");
    });
}

fn run() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(format!("127.0.0.1:{PORT}"))?;
    for connection in listener.incoming() {
        thread::spawn(|| {
            handle_connection(&connection.unwrap()).unwrap_or_else(|err| {
                println!("failed connection: {err}");
            });
        });
    }
    Ok(())
}

const CRLF: &str = "\r\n";
fn handle_connection(stream: &TcpStream) -> Result<(), Box<dyn Error>> {
    let buf_reader = BufReader::new(stream);
    let http_request: Vec<String> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();
    
    let mut request_line = http_request[0].split(' ');
    let request_type = request_line.next().unwrap();
    let request_path = format!(".{}", request_line.next().unwrap());
    
    match request_type {
        "GET" => handle_get(stream, &request_path)?,
        _ => send_file(stream, Path::new("404.html"), "HTTP/1.1 404 NOT FOUND")?
    }
    Ok(())
}

fn handle_get(stream: &TcpStream, path: &str) -> io::Result<()> {
    let canon_path = Path::new(path).canonicalize();
    if let Ok(canon_path) = canon_path {
        send_file(stream, &canon_path, "HTTP/1.1 200 OK")?;
        Ok(())
    } else {
        send_file(stream, Path::new("404.html"), "HTTP/1.1 404 NOT FOUND")?;
        Ok(())
    }
}

fn send_file(mut stream: &TcpStream, path: &Path, status_line: &str) -> io::Result<()> {
    let contents = fs::read_to_string(path)?;
    let len = contents.len();
    let response =
        format!("{status_line}{CRLF}Content-Length: {len}{CRLF}{CRLF}{contents}");

    stream.write_all(response.as_bytes())?;
    Ok(())
}
