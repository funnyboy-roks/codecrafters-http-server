use std::{collections::HashMap, io::Write, sync::Arc};

use flate2::{write::GzEncoder, Compression};
use tokio::{
    fs,
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
};

mod cli;
mod request;
mod response;

use cli::Cli;
use request::Request;
use response::Response;

async fn handle_stream(mut stream: TcpStream, cli: &Cli) -> anyhow::Result<()> {
    let req = Request::parse(&mut stream).await?;
    eprintln!("accepted new connection");
    dbg!(&req);
    match (&*req.method, &*req.path) {
        ("POST", s) if s.starts_with("/files/") => {
            let file = s.strip_prefix("/files/").unwrap();

            let mut pb = cli.directory.clone();
            pb.push(file);

            fs::write(pb, req.body).await?;
            stream.write_all(b"HTTP/1.1 201 Created\r\n\r\n").await?;
        }
        ("GET", s) if s.starts_with("/files/") => {
            let file = s.strip_prefix("/files/").unwrap();

            let mut pb = cli.directory.clone();
            pb.push(file);

            if pb.exists() {
                let res = Response::new(
                    HashMap::from([("Content-Type".into(), "application/octet-stream".into())]),
                    fs::read(pb).await.unwrap(),
                );
                stream.write_all(&res.into_bytes()).await?;
            } else {
                stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n").await?;
            }
        }
        ("GET", "/user-agent") => {
            let ua = req.headers.get("user-agent").unwrap();

            let res = Response::new(
                HashMap::from([("Content-Type".into(), "text/plain".into())]),
                ua.as_bytes().to_vec(),
            );
            dbg!(&res);
            let bytes = res.into_bytes();
            eprintln!("{}", String::from_utf8_lossy(&bytes));
            stream.write_all(&bytes).await?;
        }
        ("GET", s) if s.starts_with("/echo/") => {
            let res = if let Some(encoding) = req.headers.get("accept-encoding") {
                let encodings: Vec<_> = encoding.split(",").map(|s| s.trim().to_string()).collect();

                if encodings.contains(&"gzip".to_string()) {
                    let body = s.strip_prefix("/echo/").unwrap().as_bytes();
                    let mut compbody = Vec::new();
                    GzEncoder::new(&mut compbody, Compression::default()).write_all(body)?;

                    Response::new(
                        HashMap::from([
                            ("Content-Type".into(), "text/plain".into()),
                            ("Content-Encoding".into(), "gzip".into()),
                        ]),
                        compbody,
                    )
                } else {
                    Response::new(
                        HashMap::from([("Content-Type".into(), "text/plain".into())]),
                        s.strip_prefix("/echo/").unwrap().into(),
                    )
                }
            } else {
                Response::new(
                    HashMap::from([("Content-Type".into(), "text/plain".into())]),
                    s.strip_prefix("/echo/").unwrap().into(),
                )
            };
            stream.write_all(&res.into_bytes()).await?;
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

    let cli = Arc::new(Cli::parse());

    loop {
        let (stream, _) = listener.accept().await?;

        let cli = cli.clone();
        tokio::spawn(async move {
            match handle_stream(stream, &cli).await {
                Ok(_) => {}
                Err(e) => eprintln!("error: {:?}", e),
            }
        });
    }
}
