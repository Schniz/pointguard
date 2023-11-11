use aide::axum::ApiRouter;
use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Extension,
};
use handlebars::Handlebars;
use rust_embed::RustEmbed;
use std::path::{Path, PathBuf};

use crate::AppState;

fn handlebar_dev(handlebars: &mut Handlebars, name: &str, path: &str, from: impl Into<PathBuf>) {
    let path = from
        .into()
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .unwrap()
        .join(path);
    handlebars
        .register_template_file(name, path)
        .expect("register template file");
}

fn handlebar_prod(handlebars: &mut Handlebars, name: &str, str: &str) {
    handlebars
        .register_template_string(name, str)
        .expect("register template string");
}

/// Register a handlebars template file.
/// first argument is &mut handlebars
/// second argument is the name of the template
/// third argument is the path to the template file
macro_rules! handlebar {
    ($handlebars:expr, $name:literal, $path:literal) => {
        #[cfg(debug_assertions)]
        handlebar_dev(&mut $handlebars, $name, $path, file!());
        #[cfg(not(debug_assertions))]
        handlebar_prod($handlebars, $name, include_str!($path));
    };
}

pub(crate) fn admin_routes() -> ApiRouter<AppState> {
    let mut handlebars = Handlebars::new();
    #[cfg(not(debug_assertions))]
    {
        #[derive(RustEmbed)]
        #[folder = "views"]
        struct Views;

        for key in Views::iter() {
            let value = Views::get(&key).unwrap();
            let key = key.strip_suffix(".html.hbs").unwrap_or(&key);
            handlebars
                .register_template_string(key, &String::from_utf8_lossy(&value.data)[..])
                .expect("register template string");
        }
    }
    #[cfg(debug_assertions)]
    {
        let views = PathBuf::from(file!())
            .parent()
            .and_then(Path::parent)
            .and_then(Path::parent)
            .unwrap()
            .join("views");
        handlebars.set_dev_mode(true);
        handlebars
            .register_templates_directory(".html.hbs", views)
            .expect("load views");
    }
    let templates: Vec<_> = handlebars
        .get_templates()
        .iter()
        .map(|(name, _)| name)
        .collect();
    tracing::debug!("loaded templates: {:?}", templates);
    ApiRouter::new()
        .route("/", get(index))
        .layer(Extension(handlebars))
}

async fn index(Extension(handlebars): Extension<Handlebars<'_>>) -> impl IntoResponse {
    let body = handlebars
        .render("index", &serde_json::json!({ "title": "hi!" }))
        .expect("render index");
    Html(body)
}

pub(crate) fn attach_views_reloader(
    reloader: tower_livereload::Reloader,
) -> notify::FsEventWatcher {
    use notify::Watcher;
    let mut watcher = notify::recommended_watcher(move |_| reloader.reload()).unwrap();
    watcher
        .watch(
            &PathBuf::from(file!())
                .parent()
                .and_then(Path::parent)
                .and_then(Path::parent)
                .unwrap()
                .join("views"),
            notify::RecursiveMode::Recursive,
        )
        .unwrap();
    watcher
}
