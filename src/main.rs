use std::{
    io::{Read, Write},
    net::TcpListener,
};

use anyhow::Context;

const CRLF: &str = "\r\n";

fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    let mut buf = [0u8; 256];
    for stream in listener.incoming() {
        let mut stream = stream?;

        dbg!("foo");
        let len = stream.read(&mut buf)?;
        let mut rest = buf[..len]
            .splitn(2, |b| *b == b'\r')
            .map(|b| b.strip_prefix(b"\n").unwrap_or(b));

        let req = String::from_utf8(rest.next().context("")?.to_vec())?;
        let _rest = rest.next().unwrap();

        let (method, rest) = req.split_once(' ').context("")?;
        let (path, rest) = rest.split_once(' ').context("")?;

        dbg!(method, path, rest);

        if method == "GET" && path == "/" {
            write!(stream, "HTTP/1.1 200 OK{CRLF}{CRLF}")?;
        } else {
            write!(stream, "HTTP/1.1 404 Not Found{CRLF}{CRLF}")?;
        }
        eprintln!("accepted new connection");
    }

    Ok(())
}
