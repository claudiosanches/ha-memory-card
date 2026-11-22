use axum::Router;
use axum::http::{HeaderMap, HeaderValue, StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use rust_embed::RustEmbed;

/// The built SPA. Release builds embed it; debug builds read it from disk.
#[derive(RustEmbed)]
#[folder = "../web/dist"]
#[allow_missing = true]
struct WebDist;

const INGRESS_PATH_HEADER: &str = "x-ingress-path";

/// Everything the UI loads comes from the app itself. WebAssembly decodes
/// barcodes; Mantine sets inline styles. Ingress frames the app from Home
/// Assistant's own origin.
const CONTENT_SECURITY_POLICY: &str = "default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; \
    style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:; connect-src 'self'; \
    object-src 'none'; base-uri 'self'; form-action 'self'; frame-ancestors 'self'";

pub fn router<S: Clone + Send + Sync + 'static>() -> Router<S> {
    Router::new().fallback(serve)
}

async fn serve(uri: Uri, headers: HeaderMap) -> Response {
    let path = uri.path().trim_start_matches('/');
    if path.is_empty() || path == "index.html" {
        return index(&headers);
    }

    let Some(file) = WebDist::get(path) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    // Vite fingerprints everything under assets/, so those never change.
    let cache = if path.starts_with("assets/") {
        "public, max-age=31536000, immutable"
    } else {
        "no-cache"
    };
    (
        [
            (header::CONTENT_TYPE, mime.as_ref()),
            (header::CACHE_CONTROL, cache),
        ],
        file.data,
    )
        .into_response()
}

fn index(headers: &HeaderMap) -> Response {
    let Some(file) = WebDist::get("index.html") else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            [(header::CONTENT_SECURITY_POLICY, CONTENT_SECURITY_POLICY)],
            "Web UI not built. Run `npm run build -w web`.",
        )
            .into_response();
    };
    let html = String::from_utf8_lossy(&file.data);
    let html = match headers.get(INGRESS_PATH_HEADER).and_then(ingress_base) {
        Some(base) => inject_base(&html, &base),
        None => html.into_owned(),
    };
    (
        [
            (header::CONTENT_TYPE, "text/html; charset=utf-8"),
            (header::CACHE_CONTROL, "no-cache"),
            (header::CONTENT_SECURITY_POLICY, CONTENT_SECURITY_POLICY),
        ],
        html,
    )
        .into_response()
}

/// Turns `X-Ingress-Path` into a `<base href>` value, so relative URLs resolve
/// under the ingress prefix even if the panel URL lacks its trailing slash.
/// Only plain absolute paths pass: `//host` would point the base at another
/// site, and `..` would climb out of the prefix.
fn ingress_base(value: &HeaderValue) -> Option<String> {
    let path = value.to_str().ok()?.trim_end_matches('/');
    let mut segments = path.strip_prefix('/')?.split('/');
    let valid = segments.all(|segment| {
        !segment.is_empty()
            && segment != "."
            && segment != ".."
            && segment
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.'))
    });
    valid.then(|| format!("{path}/"))
}

fn inject_base(html: &str, base: &str) -> String {
    match html.find("<head>") {
        Some(at) => {
            let insert_at = at + "<head>".len();
            format!(
                "{}<base href=\"{base}\">{}",
                &html[..insert_at],
                &html[insert_at..]
            )
        }
        None => html.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_from_ingress_header() {
        let value = HeaderValue::from_static("/api/hassio_ingress/AbC-123_x");
        assert_eq!(
            ingress_base(&value).as_deref(),
            Some("/api/hassio_ingress/AbC-123_x/")
        );
        let trailing = HeaderValue::from_static("/api/hassio_ingress/abc/");
        assert_eq!(
            ingress_base(&trailing).as_deref(),
            Some("/api/hassio_ingress/abc/")
        );
    }

    #[test]
    fn base_rejects_unsafe_values() {
        for raw in [
            "relative/path",
            "/a\"b",
            "/a<b>",
            "https://evil.example/",
            "//evil.example",
            "//evil.example/x/",
            "/api//x",
            "/api/../x",
            "/api/./x",
            "/",
            "",
        ] {
            let value = HeaderValue::from_str(raw).unwrap();
            assert_eq!(ingress_base(&value), None, "{raw}");
        }
    }

    #[test]
    fn inject_base_after_head() {
        let html = "<!doctype html><html><head><title>x</title></head></html>";
        assert_eq!(
            inject_base(html, "/p/"),
            "<!doctype html><html><head><base href=\"/p/\"><title>x</title></head></html>"
        );
        assert_eq!(inject_base("<html></html>", "/p/"), "<html></html>");
    }
}
