mod task_loop;

use pointguard_engine_postgres as db;
use pointguard_web_api::Server;
use std::fmt::Display;

pub fn init_logging() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "pointguard=debug");
    }

    tracing_subscriber::fmt().pretty().init();
}

pub fn print_welcome_message(host: impl Display, port: impl Display) {
    use colored::Colorize;
    let orange = colored::CustomColor::new(255, 140, 0);
    let url = format!("http://{}:{}", host, port).custom_color(orange);

    eprintln!();
    eprintln!("  🏀 pointguard is ready to play at {url}");
    eprintln!();
}

#[tokio::main]
async fn main() {
    init_logging();

    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .unwrap();
    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());

    let pool = db::connect(&std::env::var("DATABASE_URL").expect("DATABASE_URL must be set"))
        .await
        .unwrap();

    tokio::spawn(task_loop::run(pool.clone()));

    Server {
        pool,
        host: host.clone(),
        port: port.clone(),
        on_bind: Box::new(move || print_welcome_message(host, port)),
    }
    .serve()
    .await;
}
