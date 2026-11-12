//! Общие помощники тестов: локальный HTTP-сервер и временные файлы.
//! Каждый тестовый файл использует лишь часть помощников.
#![allow(dead_code)]

use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::thread::{self, JoinHandle};

/// Размер буфера для чтения HTTP-запроса.
const REQUEST_BUFFER: usize = 4096;

/// Запускает сервер, который отвечает на один запрос заданным статусом и телом.
/// Возвращает базовый адрес сервера и поток, который отдаёт первую строку
/// полученного запроса (`GET /путь HTTP/1.1`).
pub fn serve_once(status: &str, body: Vec<u8>) -> (String, JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = format!("http://{}", listener.local_addr().unwrap());
    let headers = format!(
        "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut buffer = [0u8; REQUEST_BUFFER];
        let read = stream.read(&mut buffer).unwrap();
        stream.write_all(headers.as_bytes()).unwrap();
        stream.write_all(&body).unwrap();
        let request = String::from_utf8_lossy(&buffer[..read]).into_owned();
        request.lines().next().unwrap_or_default().to_string()
    });
    (address, handle)
}

/// Содержимое, похожее на PNG-файл: подпись PNG и несколько байтов.
pub fn fake_png() -> Vec<u8> {
    let mut bytes = b"\x89PNG\r\n\x1a\n".to_vec();
    bytes.extend_from_slice(b"fake image data");
    bytes
}

/// Путь к временному файлу, уникальный для процесса и имени теста.
pub fn temp_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("depgraph-test-{}-{name}", std::process::id()))
}
