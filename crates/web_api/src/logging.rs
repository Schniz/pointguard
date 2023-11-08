pub fn init() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "pointguard=debug");
    }

    tracing_subscriber::fmt().pretty().init();
}

pub fn print_welcome_message(host: &str, port: &str) {
    use colored::Colorize;
    let orange = colored::CustomColor::new(255, 140, 0);
    let url = format!("http://{}:{}", host, port).custom_color(orange);
    eprintln!();
    eprintln!("  🏀 pointguard is ready to play at {url}",);
    eprintln!();
}
