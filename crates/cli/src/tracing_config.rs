#[derive(Debug, clap::ValueEnum, Clone)]
pub enum TracingFormat {
    Pretty,
    Json,
}

impl Default for TracingFormat {
    fn default() -> Self {
        if cfg!(debug_assertions) {
            Self::Pretty
        } else {
            Self::Json
        }
    }
}

pub fn init(format: &TracingFormat) {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "pointguard=debug");
    }

    match format {
        TracingFormat::Json => tracing_subscriber::fmt()
            .json()
            .with_writer(std::io::stderr)
            .init(),
        TracingFormat::Pretty => tracing_subscriber::fmt()
            .pretty()
            .with_writer(std::io::stderr)
            .init(),
    }
}
