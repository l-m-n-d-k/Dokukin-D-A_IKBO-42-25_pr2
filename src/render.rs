//! Превращение Mermaid-текста в изображение PNG.
//!
//! Рисовать Mermaid умеет только сам Mermaid (JavaScript и браузер), поэтому
//! изображение делает внешний инструмент. Он спрятан за трейтом [`Renderer`],
//! чтобы его можно было заменить (например, на локальный `mermaid-cli`),
//! не меняя остальной код.
//!
//! Реализация [`MermaidInk`] отправляет схему публичному сервису
//! [mermaid.ink](https://mermaid.ink) по HTTP и получает в ответ PNG.
//! Схема передаётся в адресе запроса: сжимается zlib и кодируется в base64,
//! иначе даже средний граф не поместится в допустимую длину адреса.

use std::io::Write;
use std::time::Duration;

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use flate2::Compression;
use flate2::write::ZlibEncoder;
use serde_json::json;

/// Адрес сервиса по умолчанию.
pub const DEFAULT_RENDER_URL: &str = "https://mermaid.ink";

/// Переменная окружения, которой можно заменить адрес сервиса отрисовки
/// (например, на собственный сервер или на заглушку в тестах).
pub const RENDER_URL_ENV: &str = "DEPGRAPH_RENDER_URL";

/// Таймаут запроса: отрисовка большого графа на сервисе занимает время.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(60);

/// Первые байты любого PNG-файла.
const PNG_SIGNATURE: &[u8] = b"\x89PNG\r\n\x1a\n";

/// Максимальное число символов ответа сервиса, показываемое в сообщении об ошибке.
const MAX_ERROR_PREVIEW: usize = 100;

/// Что-то, что умеет превращать Mermaid-текст в PNG.
pub trait Renderer {
    /// Рисует схему `mermaid` и возвращает содержимое PNG-файла.
    ///
    /// # Ошибки
    ///
    /// Возвращает описание проблемы: сервис недоступен, схема слишком большая
    /// или ответ не является изображением PNG.
    fn render(&self, mermaid: &str) -> Result<Vec<u8>, String>;
}

/// Рисует схемы с помощью сервиса mermaid.ink.
#[derive(Debug, Clone)]
pub struct MermaidInk {
    /// Адрес сервиса без завершающего `/`.
    base_url: String,
}

impl MermaidInk {
    /// Создаёт рендерер для сервиса по адресу `base_url`.
    pub fn new(base_url: &str) -> Self {
        Self { base_url: base_url.trim_end_matches('/').to_string() }
    }
}

impl Default for MermaidInk {
    /// Рендерер для публичного сервиса [`DEFAULT_RENDER_URL`].
    fn default() -> Self {
        Self::new(DEFAULT_RENDER_URL)
    }
}

/// Кодирует схему для адреса запроса: JSON с кодом схемы → zlib → base64.
///
/// # Ошибки
///
/// Возвращает описание ошибки сжатия (практически невозможной для памяти).
pub fn encode_diagram(mermaid: &str) -> Result<String, String> {
    let payload = json!({ "code": mermaid, "mermaid": { "theme": "default" } }).to_string();
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(payload.as_bytes()).map_err(|e| format!("не удалось сжать схему: {e}"))?;
    let compressed = encoder.finish().map_err(|e| format!("не удалось сжать схему: {e}"))?;
    Ok(URL_SAFE_NO_PAD.encode(compressed))
}

impl MermaidInk {
    /// Строит адрес, по которому сервис отдаёт PNG со схемой.
    ///
    /// # Ошибки
    ///
    /// Возвращает ошибку сжатия схемы, см. [`encode_diagram`].
    pub fn image_url(&self, mermaid: &str) -> Result<String, String> {
        let code = encode_diagram(mermaid)?;
        Ok(format!("{}/img/pako:{code}?type=png&bgColor=white", self.base_url))
    }
}

/// Превращает ошибку HTTP-клиента в сообщение для пользователя.
fn describe_error(error: &ureq::Error) -> String {
    match error {
        ureq::Error::StatusCode(code) => {
            format!("сервис отрисовки вернул статус {code} (граф слишком большой или сервис занят)")
        }
        other => format!("сервис отрисовки недоступен: {other}"),
    }
}

impl Renderer for MermaidInk {
    /// Скачивает PNG со схемой с сервиса и проверяет, что это действительно PNG.
    fn render(&self, mermaid: &str) -> Result<Vec<u8>, String> {
        let agent: ureq::Agent =
            ureq::Agent::config_builder().timeout_global(Some(REQUEST_TIMEOUT)).build().into();
        let mut response =
            agent.get(&self.image_url(mermaid)?).call().map_err(|e| describe_error(&e))?;
        let body = response
            .body_mut()
            .read_to_vec()
            .map_err(|e| format!("не удалось прочитать ответ сервиса отрисовки: {e}"))?;

        if body.starts_with(PNG_SIGNATURE) {
            return Ok(body);
        }
        let preview: String =
            String::from_utf8_lossy(&body).chars().take(MAX_ERROR_PREVIEW).collect();
        Err(format!("сервис отрисовки вернул не PNG: {preview}"))
    }
}
