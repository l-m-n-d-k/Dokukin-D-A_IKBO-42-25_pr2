//! Тесты отрисовки PNG: кодирование схемы и обмен с сервисом (локальным).

mod common;

use std::io::Read;

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use common::{fake_png, serve_once};
use depgraph::render::{MermaidInk, Renderer, encode_diagram};
use flate2::read::ZlibDecoder;

/// Раскодирует схему обратно: base64 → zlib → JSON → поле `code`.
fn decode_diagram(encoded: &str) -> String {
    let compressed = URL_SAFE_NO_PAD.decode(encoded).unwrap();
    let mut json = String::new();
    ZlibDecoder::new(compressed.as_slice()).read_to_string(&mut json).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    value["code"].as_str().unwrap().to_string()
}

#[test]
fn encoded_diagram_can_be_decoded_back() {
    let text = "graph TD\n    n0[\"A@1.0.0\"]\n    n0 --> n0\n";
    assert_eq!(decode_diagram(&encode_diagram(text).unwrap()), text);
}

#[test]
fn encoded_diagram_is_url_safe_and_shorter_for_big_graphs() {
    let text = "graph TD\n".to_string() + &"    n0 --> n1\n".repeat(500);
    let encoded = encode_diagram(&text).unwrap();
    assert!(encoded.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'));
    assert!(encoded.len() < text.len());
}

#[test]
fn image_url_points_to_png_endpoint() {
    let url = MermaidInk::new("http://example.test/").image_url("graph TD\n").unwrap();
    assert!(url.starts_with("http://example.test/img/pako:"), "{url}");
    assert!(url.ends_with("?type=png&bgColor=white"), "{url}");
}

#[test]
fn png_is_returned_from_the_service() {
    let (address, server) = serve_once("200 OK", fake_png());
    let png = MermaidInk::new(&address).render("graph TD\n    n0[\"A\"]\n").unwrap();
    assert_eq!(png, fake_png());
    assert!(server.join().unwrap().starts_with("GET /img/pako:"));
}

#[test]
fn non_png_answer_is_an_error() {
    let (address, _server) = serve_once("200 OK", b"<html>oops</html>".to_vec());
    let error = MermaidInk::new(&address).render("graph TD\n").unwrap_err();
    assert!(error.contains("не PNG") && error.contains("oops"), "{error}");
}

#[test]
fn error_status_is_reported() {
    let (address, _server) = serve_once("414 URI Too Long", Vec::new());
    let error = MermaidInk::new(&address).render("graph TD\n").unwrap_err();
    assert!(error.contains("414"), "{error}");
}

#[test]
fn unreachable_service_is_reported() {
    // Порт 1 никто не слушает.
    let error = MermaidInk::new("http://127.0.0.1:1").render("graph TD\n").unwrap_err();
    assert!(error.contains("недоступен"), "{error}");
}
