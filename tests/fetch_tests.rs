//! Тесты загрузки данных с локального HTTP-сервера вместо реального npm.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

use depgraph::graph::{DependencySource, build_graph};
use depgraph::npm::{NpmRepository, fetch_manifest};

/// Размер буфера для чтения HTTP-запроса.
const REQUEST_BUFFER: usize = 4096;

/// Ответ сервера: статус и тело.
type Reply = (&'static str, &'static str);

/// Поток сервера; по завершении отдаёт пути полученных запросов.
type Server = thread::JoinHandle<Vec<String>>;

/// Читает один HTTP-запрос из соединения, отвечает на него по таблице
/// маршрутов и возвращает путь из запроса. Неизвестный путь даёт 404.
fn answer(mut stream: TcpStream, routes: &HashMap<&str, Reply>) -> String {
    let mut buffer = [0u8; REQUEST_BUFFER];
    let read = stream.read(&mut buffer).unwrap();
    let request = String::from_utf8_lossy(&buffer[..read]).into_owned();
    let path = request.split_whitespace().nth(1).unwrap_or_default().to_string();

    let not_found = ("404 Not Found", "{}");
    let (status, body) = routes.get(path.as_str()).copied().unwrap_or(not_found);
    let length = body.len();
    let headers = format!("Content-Length: {length}\r\nConnection: close");
    let response = format!("HTTP/1.1 {status}\r\n{headers}\r\n\r\n{body}");
    stream.write_all(response.as_bytes()).unwrap();
    path
}

/// Запускает сервер, который обслуживает `count` запросов.
/// Возвращает базовый адрес сервера и поток с путями полученных запросов.
fn serve(routes: &[(&'static str, Reply)], count: usize) -> (String, Server) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = format!("http://{}", listener.local_addr().unwrap());
    let routes: HashMap<&str, Reply> = routes.iter().copied().collect();
    let handle = thread::spawn(move || {
        let connections = listener.incoming().take(count);
        connections.map(|stream| answer(stream.unwrap(), &routes)).collect()
    });
    (address, handle)
}

/// Документ пакета `app`: версии 1.0.0 и 1.1.0, зависимости от `lib` и `util`.
const APP: &str = r#"{
    "dist-tags": {"latest": "1.1.0"},
    "versions": {
        "1.0.0": {"dependencies": {"lib": "^1.0.0"}},
        "1.1.0": {"dependencies": {"lib": "^1.0.0", "util": "~2.0.0"}}
    }
}"#;

/// Документ пакета `lib`: зависит от `app`, образуя цикл.
const LIB: &str = r#"{
    "versions": {
        "1.0.0": {"dependencies": {"app": "*"}},
        "1.4.0": {"dependencies": {"app": "*"}},
        "2.0.0": {}
    }
}"#;

/// Документ пакета `util`.
const UTIL: &str = r#"{"versions": {"2.0.0": {}, "2.0.7": {}, "2.1.0": {}}}"#;

#[test]
fn graph_is_built_from_repository_url() {
    let ok = "200 OK";
    let routes = [("/app", (ok, APP)), ("/lib", (ok, LIB)), ("/util", (ok, UTIL))];
    let (address, server) = serve(&routes, 3);

    let mut repository = NpmRepository::new(&address);
    let graph = build_graph(&mut repository, "app", "1.1.0").unwrap();

    let found: Vec<_> = graph.nodes.iter().map(|n| format!("{}@{}", n.name, n.version)).collect();
    assert_eq!(found, ["app@1.1.0", "lib@1.4.0", "util@2.0.7"]);
    assert_eq!(graph.node("lib").unwrap().dependencies, ["app"]);
    assert_eq!(server.join().unwrap(), ["/app", "/lib", "/util"]);
}

#[test]
fn root_version_is_taken_from_config() {
    let (address, _server) = serve(&[("/app", ("200 OK", APP))], 1);
    let mut repository = NpmRepository::new(&address);
    let package = repository.package("app", "1.0.0").unwrap();
    assert_eq!(package.version, "1.0.0");
    assert_eq!(package.dependencies.len(), 1);
}

#[test]
fn missing_root_version_is_an_error() {
    let (address, _server) = serve(&[("/app", ("200 OK", APP))], 1);
    let mut repository = NpmRepository::new(&address);
    let error = repository.package("app", "9.9.9").unwrap_err();
    assert!(error.contains("9.9.9"), "{error}");
}

#[test]
fn missing_package_is_reported_with_its_name() {
    let (address, _server) = serve(&[("/app", ("200 OK", APP))], 2);
    let mut repository = NpmRepository::new(&address);
    let error = build_graph(&mut repository, "app", "1.0.0").unwrap_err();
    assert!(error.contains("'lib'") && error.contains("не найден"), "{error}");
}

#[test]
fn not_found_is_reported_as_missing_package() {
    let (address, _server) = serve(&[], 1);
    let error = fetch_manifest(&format!("{address}/demo")).unwrap_err();
    assert!(error.contains("не найден"), "{error}");
}

#[test]
fn server_error_status_is_reported() {
    let (address, _server) = serve(&[("/demo", ("500 Internal Server Error", "oops"))], 1);
    let error = fetch_manifest(&format!("{address}/demo")).unwrap_err();
    assert!(error.contains("500"), "{error}");
}

#[test]
fn unreachable_server_is_reported() {
    // Порт 1 никто не слушает.
    let error = fetch_manifest("http://127.0.0.1:1/demo").unwrap_err();
    assert!(error.contains("не удалось подключиться"), "{error}");
}

#[test]
fn invalid_json_from_server_is_an_error() {
    let (address, _server) = serve(&[("/app", ("200 OK", "<html>not json</html>"))], 1);
    let error = NpmRepository::new(&address).package("app", "1.0.0").unwrap_err();
    assert!(error.contains("JSON"), "{error}");
}
