//! Тесты загрузки данных с локальным HTTP-сервером вместо реального npm.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

use depgraph::config::Config;
use depgraph::error::AppError;
use depgraph::npm::{fetch_dependencies, fetch_manifest};

/// Размер буфера для чтения HTTP-запроса.
const REQUEST_BUFFER: usize = 4096;

/// Запускает сервер, который отвечает на один запрос заданным статусом
/// и телом. Возвращает базовый адрес сервера и поток с полученным запросом.
fn serve_once(status: &str, body: &str) -> (String, thread::JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = format!("http://{}", listener.local_addr().unwrap());
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut buffer = [0u8; REQUEST_BUFFER];
        let read = stream.read(&mut buffer).unwrap();
        stream.write_all(response.as_bytes()).unwrap();
        String::from_utf8_lossy(&buffer[..read]).into_owned()
    });
    (address, handle)
}

/// Конфигурация, указывающая на локальный сервер.
fn config_for(repository: String) -> Config {
    Config {
        package_name: "demo".to_string(),
        repository,
        test_mode: false,
        version: "1.2.3".to_string(),
        output_file: "graph.png".to_string(),
    }
}

#[test]
fn dependencies_are_fetched_from_repository_url() {
    let body = r#"{"dependencies": {"left": "^1.0.0", "right": "2.x"}}"#;
    let (address, server) = serve_once("200 OK", body);

    let deps = fetch_dependencies(&config_for(address)).unwrap();

    assert_eq!(deps.len(), 2);
    assert_eq!(deps["left"], "^1.0.0");
    assert!(server.join().unwrap().starts_with("GET /demo/1.2.3 HTTP/1.1"));
}

#[test]
fn not_found_is_reported_as_missing_package() {
    let (address, _server) = serve_once("404 Not Found", "{}");
    let error = fetch_manifest(&format!("{address}/demo/1.2.3")).unwrap_err();
    assert!(error.contains("не найдены"), "{error}");
}

#[test]
fn server_error_status_is_reported() {
    let (address, _server) = serve_once("500 Internal Server Error", "oops");
    let error = fetch_manifest(&format!("{address}/demo/1.2.3")).unwrap_err();
    assert!(error.contains("500"), "{error}");
}

#[test]
fn unreachable_server_is_reported() {
    // Порт 1 никто не слушает.
    let error = fetch_manifest("http://127.0.0.1:1/demo/1.2.3").unwrap_err();
    assert!(error.contains("не удалось подключиться"), "{error}");
}

#[test]
fn invalid_json_from_server_is_an_error() {
    let (address, _server) = serve_once("200 OK", "<html>not json</html>");
    let result = fetch_dependencies(&config_for(address));
    assert!(matches!(result, Err(AppError::Npm(_))), "{result:?}");
}

#[test]
fn test_mode_is_not_supported_yet() {
    let mut config = config_for("tests/fixtures/test_repo.txt".to_string());
    config.test_mode = true;
    assert!(matches!(fetch_dependencies(&config), Err(AppError::Unsupported(_))));
}
