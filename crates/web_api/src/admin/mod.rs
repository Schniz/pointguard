mod views;

use crate::AppState;
use aide::axum::ApiRouter;
use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Extension,
};
use handlebars::Handlebars;
use std::path::{Path, PathBuf};

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
