use std::collections::HashMap;
use std::io::Write;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Response {
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl Response {
    pub fn new(headers: HashMap<String, String>, body: Vec<u8>) -> Self {
        Self { headers, body }
    }

    pub fn into_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        write!(out, "HTTP/1.1 200 OK\r\n").unwrap();

        for (k, v) in &self.headers {
            write!(out, "{}: {}\r\n", k, v).unwrap();
            if self.body.len() > 0 {
                write!(out, "Content-Length: {}\r\n", self.body.len()).unwrap();
            }
        }
        write!(out, "\r\n").unwrap();
        out.extend_from_slice(&self.body);

        out
    }
}
