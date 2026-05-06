use axum::{routing::any, Router};
use tracing::info;
use tracing_subscriber::EnvFilter;

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

    let app = Router::new().fallback(any(handler));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn handler(req: axum::extract::Request) -> &'static str {
    info!(method = %req.method(), uri = %req.uri(), headers = ?req.headers(), "request");
    "ok"
}
