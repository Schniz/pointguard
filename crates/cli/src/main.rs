mod task_loop;

use clap::{Parser, Subcommand};
use futures::future::FutureExt;
use pointguard_engine_postgres as db;
use pointguard_web_api::Server;
use std::fmt::Display;

pub fn init_logging() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "pointguard=debug");
    }

    tracing_subscriber::fmt()
        .pretty()
        .with_writer(std::io::stderr)
        .init();
}

pub fn print_welcome_message(host: impl Display, port: impl Display) {
    use colored::Colorize;
    let orange = colored::CustomColor::new(255, 140, 0);
    let url = format!("http://{}:{}", host, port).custom_color(orange);

    eprintln!();
    eprintln!("  🏀 pointguard is ready to play at {url}");
    eprintln!();
}

#[derive(Parser, Debug)]
struct Cli {
    #[clap(subcommand)]
    subcommand: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Run the web server
    Serve(Serve),

    /// Print the OpenAPI spec
    #[clap(name = "openapi-spec")]
    OpenApiSpec(OpenApiSpec),
}

#[derive(Parser, Debug)]
struct Serve {
    #[clap(long, env = "DATABASE_URL")]
    database_url: String,

    #[clap(long, env = "HOST", default_value = "127.0.0.1")]
    host: String,

    #[clap(long, env = "PORT", default_value = "8080")]
    port: u16,

    #[clap(long = "migrate")]
    should_migrate: bool,

    #[clap(long = "database-schema", env = "DATABASE_SCHEMA")]
    schema: Option<String>,
}

impl Serve {
    async fn call(self) {
        let db_options = db::DbOptions {
            schema: self.schema,
        };
        let pool = db::connect(&self.database_url, &db_options).await.unwrap();

        if self.should_migrate {
            db::migrate(&pool, &db_options)
                .await
                .expect("running migrations");
        }

        let termination = shutdown_signal().shared();

        let task_loop = task_loop::run(pool.clone(), termination.clone());

        let serving = Server {
            pool,
            host: self.host,
            port: self.port,
            on_bind: Box::new(|host, port| print_welcome_message(host, port)),
        }
        .serve(termination);

        tokio::join!(task_loop, serving);

        tracing::info!("goodbye!");
    }
}

#[tokio::main]
async fn main() {
    init_logging();

    match Cli::parse().subcommand {
        Command::Serve(serve) => serve.call().await,
        Command::OpenApiSpec(spec) => spec.call(),
    }
}

#[derive(Parser, Debug)]
struct OpenApiSpec {
    #[clap(long)]
    pretty: bool,
}

impl OpenApiSpec {
    fn call(self) {
        let mut spec = pointguard_web_api::openapi::new();
        let _ = pointguard_web_api::api_router(&mut spec);
        if self.pretty {
            serde_json::to_writer_pretty(std::io::stdout(), &spec)
                .expect("writing OpenAPI spec to stdout");
        } else {
            serde_json::to_writer(std::io::stdout(), &spec)
                .expect("writing OpenAPI spec to stdout");
        }
    }
}

async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("signal received, starting graceful shutdown");
}
