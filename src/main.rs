use std::net::SocketAddr;

use axum::{
    body::Body,
    extract::{ConnectInfo, Request, State},
    http::{header::HOST, HeaderName, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::{any, get},
    Router,
};

const X_FORWARDED_FOR: HeaderName = HeaderName::from_static("x-forwarded-for");
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

#[derive(Clone)]
struct AppState {
    client: reqwest::Client,
    origin: String,
    host: HeaderValue,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_line_number(true)
        .with_file(true)
        .init();

    let origin = std::env::var("ORIGIN_URL")
        .expect("ORIGIN_URL must be set")
        .trim_end_matches('/')
        .to_string();

    let parsed = reqwest::Url::parse(&origin).expect("ORIGIN_URL must be a valid URL");
    let host_str = match parsed.port() {
        Some(p) => format!(
            "{}:{}",
            parsed.host_str().expect("ORIGIN_URL must have a host"),
            p
        ),
        None => parsed
            .host_str()
            .expect("ORIGIN_URL must have a host")
            .to_string(),
    };
    let host = HeaderValue::from_str(&host_str).expect("ORIGIN_URL host must be valid");

    let state = AppState {
        client: reqwest::Client::new(),
        origin: origin.clone(),
        host,
    };

    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .fallback(any(handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    info!(origin = %origin, addr = %listener.local_addr().unwrap(), "listening");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

async fn healthz() -> &'static str {
    "ok"
}

async fn readyz() -> &'static str {
    "ok"
}

async fn handler(
    State(state): State<AppState>,
    ConnectInfo(remote): ConnectInfo<SocketAddr>,
    req: Request,
) -> Response {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let mut headers = req.headers().clone();
    info!(method = %method, uri = %uri, remote = %remote, headers = ?headers, "request");
    headers.insert(HOST, state.host.clone());
    if !headers.contains_key(&X_FORWARDED_FOR) {
        if let Ok(v) = HeaderValue::from_str(&remote.ip().to_string()) {
            headers.insert(X_FORWARDED_FOR, v);
        }
    }

    let path_and_query = uri
        .path_and_query()
        .map(|pq| pq.as_str())
        .unwrap_or(uri.path());
    let target = format!("{}{}", state.origin, path_and_query);

    let body_bytes = match axum::body::to_bytes(req.into_body(), usize::MAX).await {
        Ok(b) => b,
        Err(e) => {
            error!(error = %e, "failed to read request body");
            return (StatusCode::BAD_REQUEST, "failed to read body").into_response();
        }
    };

    let upstream = match state
        .client
        .request(method, &target)
        .headers(headers)
        .body(body_bytes)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            error!(error = %e, target = %target, "upstream request failed");
            return (StatusCode::BAD_GATEWAY, "upstream request failed").into_response();
        }
    };

    let status = upstream.status();
    let resp_headers = upstream.headers().clone();
    let stream = upstream.bytes_stream();

    let mut builder = Response::builder().status(status);
    if let Some(h) = builder.headers_mut() {
        *h = resp_headers;
    }
    builder.body(Body::from_stream(stream)).unwrap_or_else(|e| {
        error!(error = %e, "failed to build response");
        (StatusCode::INTERNAL_SERVER_ERROR, "response build failed").into_response()
    })
}
