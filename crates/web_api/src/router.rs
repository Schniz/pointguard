use crate::admin::admin_routes;
use crate::AppState;
use aide::{
    axum::{
        routing::{get, get_with, post},
        ApiRouter, IntoApiResponse,
    },
    openapi::OpenApi,
    redoc::Redoc,
};
use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Redirect, Sse},
    Extension, Json,
};
use db::PaginationCursor;
use flume::{Receiver, Sender};
use futures::StreamExt;
use pointguard_engine_postgres as db;
use pointguard_types::Event;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

async fn get_finished_tasks(
    State(state): State<AppState>,
    Query(query): Query<PaginationCursor>,
) -> impl IntoApiResponse {
    let finished_tasks = db::finished_tasks(&state.db, &query)
        .await
        .expect("finished tasks");
    Json(finished_tasks)
}

async fn get_enqueued_tasks(State(state): State<AppState>) -> impl IntoApiResponse {
    let enqueued_tasks = db::enqueued_tasks(&state.db).await.expect("enqueued tasks");
    Json(enqueued_tasks)
}

async fn serve_api(Extension(api): Extension<OpenApi>) -> impl IntoApiResponse {
    Json(api)
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
struct CancelTaskParams {
    id: i64,
}

async fn cancel_task(
    State(state): State<AppState>,
    axum::extract::Path(path): axum::extract::Path<CancelTaskParams>,
) -> impl IntoApiResponse {
    let _task = db::cancel_task(&state.db, path.id)
        .await
        .expect("cancel task");
    Redirect::to("/api/v1/tasks/enqueued")
}

async fn unshift_task(
    State(state): State<AppState>,
    axum::extract::Path(path): axum::extract::Path<CancelTaskParams>,
) -> impl IntoApiResponse {
    let _task = db::unshift_job(&state.db, path.id)
        .await
        .expect("unshift task");
    Redirect::to("/api/v1/tasks/enqueued")
}

#[tracing::instrument(skip_all, fields(%new_task.job_name))]
async fn post_tasks(
    Extension(event_tx): Extension<Sender<Event>>,
    State(state): State<AppState>,
    Json(new_task): Json<NewTaskBody>,
) -> impl IntoApiResponse {
    let id = db::enqueue(
        &state.db,
        &db::NewTask {
            job_name: new_task.job_name,
            max_retries: new_task.max_retries.map(|x| x as i32),
            data: new_task.data.unwrap_or_default(),
            endpoint: new_task.endpoint.to_string(),
            name: new_task.name.unwrap_or_else(generate_nanoid),
            run_at: new_task.run_at,
        },
    )
    .await
    .expect("enqueue task");

    event_tx
        .send_async(Event::TaskEnqueued)
        .await
        .expect("send task");

    Json(id)
}

async fn events(Extension(event_rx): Extension<Receiver<Event>>) -> impl IntoApiResponse {
    Sse::new(event_rx.into_stream().map(|x| {
        axum::response::sse::Event::default()
            .event("update")
            .json_data(x)
    }))
    .into_response()
}

pub fn api_router(api: &mut OpenApi) -> axum::Router<AppState> {
    ApiRouter::new()
        .route("/api", Redoc::new("/api/openapi.json").axum_route())
        .route("/api/openapi.json", get(serve_api))
        .nest("/", admin_routes())
        .api_route(
            "/api/v1/events",
            get_with(events, |r| {
                r.summary("/api/v1/events")
                    .description("get realtime events. This is a server sent event stream, but I can't figure out how to document it in openapi.")
                    .tag("internal")
                    .response::<200, Json<Event>>()
            }),
        )
        .api_route("/api/v1/version", get(stub))
        .api_route("/api/v1/tasks", post(post_tasks))
        .api_route("/api/v1/tasks/:id/cancel", post(cancel_task))
        .api_route("/api/v1/tasks/:id/unshift", post(unshift_task))
        .api_route("/api/v1/tasks/enqueued", get(get_enqueued_tasks))
        .api_route("/api/v1/tasks/finished", get(get_finished_tasks))
        .finish_api_with(api, |api| api.default_response::<String>())
}

fn generate_nanoid() -> String {
    nanoid::nanoid!()
}

async fn stub() -> impl IntoApiResponse {
    "Hello, world!"
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct NewTaskBody {
    /// A name for the task. If not provided, a random name will be generated.
    /// This is useful to throttle tasks of the same type.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    /// The job name. This is used to know which function to invoke.
    job_name: String,
    /// The data that will be passed on execution.
    data: Option<serde_json::Value>,
    /// The pointguard endpoint that'll be invoked
    endpoint: url::Url,
    /// When to run the task. If not provided, it'll run as soon as possible.
    #[serde(skip_serializing_if = "Option::is_none")]
    run_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_retries: Option<usize>,
}
