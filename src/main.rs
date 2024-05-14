use std::{
    collections::HashMap,
    io::{Read, Write},
    net::TcpListener,
};

use anyhow::Context;

const CRLF: &str = "\r\n";

#[derive(Debug, Clone, Eq, PartialEq)]
struct Request {
    method: String,
    path: String,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

impl Request {
    pub fn read_until<R>(r: &mut R, delim: u8) -> anyhow::Result<String>
    where
        R: Read,
    {
        let mut bytes = r.bytes();
        let mut out = String::with_capacity(5);
        loop {
            let byte = bytes.next().context("foo")??;

            if byte == delim {
                break;
            }

            out.push(byte as char);
        }

        Ok(out)
    }

    pub fn parse<R>(r: &mut R) -> anyhow::Result<Request>
    where
        R: Read,
    {
        let method = Self::read_until(r, b' ')?;
        let path = Self::read_until(r, b' ')?;

        Self::read_until(r, b'\r')?;
        let mut buf = [0u8; 1];
        r.read_exact(&mut buf)?;
        assert_eq!(buf[0], b'\n');

        let mut headers = HashMap::new();

        loop {
            let s = Self::read_until(r, b'\n')?;
            let s = s.strip_suffix('\r').unwrap();

            if s == "" {
                break;
            }

            let (k, v) = s.split_once(':').unwrap();

            let v = v.strip_prefix(' ').unwrap_or(v);

            headers.insert(k.into(), v.into());
        }

        let body = Vec::new();
        // let mut buf = [0u8; 256];
        // loop {
        //     let len = r.read(&mut buf)?;

        //     body.extend_from_slice(&buf[..len]);

        //     if len < buf.len() {
        //         break;
        //     }
        // }

        dbg!(&body);

        Ok(Request {
            method,
            path,
            headers,
            body,
        })
    }
}

fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    // let mut buf = [0u8; 256];
    for stream in listener.incoming() {
        let mut stream = match stream {
            Ok(stream) => stream,
            Err(e) => {
                eprintln!("Error: {:?}", e);
                continue;
            }
        };

        let req = Request::parse(&mut stream)?;
        eprintln!("accepted new connection");
        dbg!(&req);
        match (&*req.method, &*req.path) {
            ("GET", s) if s.starts_with("/echo/") => {
                write!(
                    stream,
                    "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 3\r\n\r\n{}",
                    s.strip_prefix("/echo/").unwrap()
                )?;
            }
            ("GET", "/") => {
                write!(stream, "HTTP/1.1 200 OK{CRLF}{CRLF}")?;
            }
            _ => {
                write!(stream, "HTTP/1.1 404 Not Found{CRLF}{CRLF}")?;
            }
        }
    }

    Ok(())
}
