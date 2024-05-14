use std::collections::HashMap;

use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

#[derive(Debug, Clone, Eq, PartialEq)]
struct Request {
    method: String,
    path: String,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

impl Request {
    pub async fn read_until<R>(r: &mut R, delim: u8) -> anyhow::Result<String>
    where
        R: AsyncRead + Unpin,
    {
        let mut out = String::with_capacity(5);
        let mut buf = [0u8; 1];
        loop {
            let l = r.read_exact(&mut buf).await?;
            assert_eq!(l, 1);
            let byte = buf[0];

            if byte == delim {
                break;
            }

            out.push(byte as char);
        }

        Ok(out)
    }

    pub async fn parse<R>(r: &mut R) -> anyhow::Result<Request>
    where
        R: AsyncRead + std::marker::Unpin,
    {
        let method = Self::read_until(r, b' ').await?;
        let path = Self::read_until(r, b' ').await?;

        Self::read_until(r, b'\r').await?;
        let mut buf = [0u8; 1];
        r.read_exact(&mut buf).await?;
        assert_eq!(buf[0], b'\n');

        let mut headers = HashMap::new();

        loop {
            let s = Self::read_until(r, b'\n').await?;
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

async fn handle_stream(mut stream: TcpStream) -> anyhow::Result<()> {
    let req = Request::parse(&mut stream).await?;
    eprintln!("accepted new connection");
    dbg!(&req);
    match (&*req.method, &*req.path) {
        ("GET", "/user-agent") => {
            let ua = req.headers.get("User-Agent").unwrap();
            stream.write_all(
                    format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                        ua.len(),
                        ua
                    )
                    .as_bytes(),
                ).await?;
        }
        ("GET", s) if s.starts_with("/echo/") => {
            stream.write_all(
                    format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                        s.len() - 6,
                        s.strip_prefix("/echo/").unwrap()
                    )
                    .as_bytes(),
                ).await?;
        }
        ("GET", "/") => {
            stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n").await?;
        }
        _ => {
            stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n").await?;
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4221").await.unwrap();

    // let mut buf = [0u8; 256];
    loop {
        let (stream, _) = listener.accept().await?;

        tokio::spawn(async move {
            match handle_stream(stream).await {
                Ok(_) => {}
                Err(e) => eprintln!("error: {:?}", e),
            }
        });
    }
}
