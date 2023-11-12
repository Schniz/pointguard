mod file_hash_helper;
mod public;
mod views;

use crate::AppState;
use aide::axum::ApiRouter;
use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Extension,
};
use handlebars::Handlebars;
use std::{
    path::{Path, PathBuf},
    time::Duration,
};

pub(crate) fn admin_routes() -> ApiRouter<AppState> {
    let mut handlebars = Handlebars::new();
    views::load(&mut handlebars);

    let templates: Vec<_> = handlebars
        .get_templates()
        .iter()
        .map(|(name, _)| name)
        .collect();
    tracing::debug!("loaded templates: {:?}", templates);
    ApiRouter::new()
        .route("/", get(index))
        .layer(Extension(handlebars))
        .nest_service("/assets/", public::serve())
}

async fn index(Extension(handlebars): Extension<Handlebars<'_>>) -> impl IntoResponse {
    let body = handlebars
        .render("index", &serde_json::json!({ "title": "hi!" }))
        .expect("render index");
    Html(body)
}

pub(crate) fn attach_views_reloader(
    reloader: tower_livereload::Reloader,
) -> notify_debouncer_mini::Debouncer<notify::FsEventWatcher> {
    use notify_debouncer_mini::new_debouncer;
    let mut debouncer = new_debouncer(Duration::from_millis(50), move |_res| {
        reloader.reload();
    })
    .unwrap();
    let root = PathBuf::from(file!())
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .unwrap()
        .to_path_buf();
    debouncer
        .watcher()
        .watch(&root.join("views"), notify::RecursiveMode::Recursive)
        .unwrap();
    debouncer
        .watcher()
        .watch(&root.join("public"), notify::RecursiveMode::Recursive)
        .unwrap();
    debouncer
}
