use std::collections::HashMap;

use tokio::io::{AsyncRead, AsyncReadExt};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
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