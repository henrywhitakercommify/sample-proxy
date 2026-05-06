use tracing::info;
use tracing_subscriber::EnvFilter;

fn main() {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_line_number(true)
        .with_file(true)
        .init();
    info!("here");
}
