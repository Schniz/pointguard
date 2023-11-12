mod file_hash_helper;
mod public;
mod views;

use crate::{events::EnqueuedTasks, AppState};
use aide::axum::ApiRouter;
use axum::{
    extract::State,
    response::{
        sse::{Event, KeepAlive},
        Html, IntoResponse, Sse,
    },
    routing::get,
    Extension,
};
use futures::StreamExt;
use handlebars::Handlebars;
use pointguard_engine_postgres as db;
use std::{
    convert::Infallible,
    path::{Path, PathBuf},
    sync::Arc,
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
        .route("/admin/events", get(dashboard_events))
        .layer(Extension(handlebars))
        .nest_service("/assets/", public::serve())
}

async fn dashboard_events(
    Extension(enqueued_tasks): Extension<Arc<EnqueuedTasks>>,
) -> impl IntoResponse {
    let stream = enqueued_tasks
        .rx
        .clone()
        .into_stream()
        .map(|_| Ok::<_, Infallible>(Event::default().data("hi").event("enqueued")));
    Sse::new(stream).keep_alive(KeepAlive::default())
}

async fn index(
    State(state): State<AppState>,
    Extension(handlebars): Extension<Handlebars<'_>>,
) -> impl IntoResponse {
    let finished_tasks = db::finished_tasks(&state.db).await.expect("finished tasks");
    let body = handlebars
        .render(
            "index",
            &serde_json::json!({ "title": "Home", "tasks": finished_tasks }),
        )
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
