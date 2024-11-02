use std::{
    collections::HashMap,
    io::{BufRead, BufReader, BufWriter, Read, Write},
    net::{TcpListener, TcpStream},
};

use anyhow::{bail, Ok, Result};

#[derive(Debug)]
enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    HEAD,
    OPTIONS,
    CONNECT,
    TRACE,
    UNKNOWN,
}

impl From<&str> for HttpMethod {
    fn from(s: &str) -> Self {
        match s {
            "GET" => HttpMethod::GET,
            "POST" => HttpMethod::POST,
            "PUT" => HttpMethod::PUT,
            "DELETE" => HttpMethod::DELETE,
            "PATCH" => HttpMethod::PATCH,
            "HEAD" => HttpMethod::HEAD,
            "OPTIONS" => HttpMethod::OPTIONS,
            "CONNECT" => HttpMethod::CONNECT,
            "TRACE" => HttpMethod::TRACE,
            _ => HttpMethod::UNKNOWN,
        }
    }
}

impl ToString for HttpMethod {
    fn to_string(&self) -> String {
        match self {
            HttpMethod::GET => "GET".to_string(),
            HttpMethod::POST => "POST".to_string(),
            HttpMethod::PUT => "PUT".to_string(),
            HttpMethod::DELETE => "DELETE".to_string(),
            HttpMethod::PATCH => "PATCH".to_string(),
            HttpMethod::HEAD => "HEAD".to_string(),
            HttpMethod::OPTIONS => "OPTIONS".to_string(),
            HttpMethod::CONNECT => "CONNECT".to_string(),
            HttpMethod::TRACE => "TRACE".to_string(),
            HttpMethod::UNKNOWN => "UNKNOWN".to_string(),
        }
    }
}

#[derive(Debug)]
struct HttpRequest<'a> {
    method: HttpMethod,
    path: &'a str,
    version: &'a str,
    headers: HashMap<String, Vec<String>>,
    body: Option<Vec<u8>>,
}

fn handle_client(stream: &mut TcpStream) -> Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut writer = BufWriter::new(stream.try_clone()?);

    loop {
        let (method, path, version) = {
            let mut buf = String::new();
            match reader.read_line(&mut buf) {
                Result::Ok(0) => return Ok(()),
                Result::Ok(_) => (),
                Err(e) => bail!(e),
            }
            match buf.trim().split_whitespace().collect::<Vec<_>>().as_slice() {
                [method, path, version] => {
                    (method.to_string(), path.to_string(), version.to_string())
                }
                _ => bail!("Invalid request"),
            }
        };

        let headers = {
            let mut headers = HashMap::<String, Vec<String>>::new();
            loop {
                let mut buf = String::new();
                let n = reader.read_line(&mut buf)?;
                if n == 0 {
                    return Ok(());
                }
                let buf = buf.trim();
                if buf.is_empty() {
                    break;
                }

                if let Some((key, value)) = buf.split_once(":") {
                    let key = key.trim().to_string();
                    let value = value.trim().to_string();
                    headers.entry(key).or_insert_with(Vec::new).push(value);
                }
            }
            headers
        };

        let body = {
            if let Some(content_length) = headers.get("Content-Length") {
                let content_length = content_length[0].parse::<usize>()?;
                let mut body = vec![0; content_length];
                reader.read_exact(&mut body)?;
                Some(body)
            } else {
                None
            }
        };

        let req = HttpRequest {
            method: method.as_str().into(),
            path: path.as_str(),
            version: version.as_str(),
            headers,
            body,
        };

        println!("{:?}", req);

        let content = format!("{} {}\n", req.method.to_string(), req.path);
        let content = content.as_bytes();
        writer.write_all(b"HTTP/1.1 200 OK\r\n")?;
        writer.write_all(b"Content-Type: text/plain\r\n")?;
        writer.write_all(format!("Content-Length: {}\r\n", content.len()).as_bytes())?;
        writer.write_all(b"\r\n")?;
        writer.write_all(content)?;
        writer.flush()?;
    }
}

fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4567")?;

    // accept connections and process them serially
    for stream in listener.incoming() {
        handle_client(&mut stream?)?;
    }
    Ok(())
}
