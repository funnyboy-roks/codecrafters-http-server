use std::{io::Write, net::TcpListener};

const CRLF: &str = "\r\n";

fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                write!(stream, "HTTP/1.1 200 OK{CRLF}{CRLF}")?;
                eprintln!("accepted new connection");
            }
            Err(e) => {
                eprintln!("error: {}", e);
            }
        }
    }

    Ok(())
}
