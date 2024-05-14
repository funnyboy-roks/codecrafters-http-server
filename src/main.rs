use std::{collections::HashMap, path::PathBuf, sync::Arc};

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
        ("GET", s) if s.starts_with("/files/") => {
            let file = s.strip_prefix("/files/").unwrap();

            let mut pb = cli.directory.clone();
            pb.push(file);

            let res = Response::new(
                HashMap::from([("Content-Type".into(), "application/octet-stream".into())]),
                fs::read(pb).await.unwrap(),
            );
            dbg!(&res);
            let bytes = res.into_bytes();
            eprintln!("{}", String::from_utf8_lossy(&bytes));
            stream.write_all(&bytes).await?;
        }
        ("GET", "/user-agent") => {
            let ua = req.headers.get("User-Agent").unwrap();

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
