mod admin;
mod events;

use std::sync::Arc;

use admin::admin_routes;
use aide::{
    axum::{
        routing::{get, post},
        ApiRouter, IntoApiResponse,
    },
    openapi::{Info, OpenApi},
    redoc::Redoc,
};
use axum::{extract::State, response::Redirect, Extension, Json};
use db::postgres::PgPool;
use events::EnqueuedTasks;
use pointguard_engine_postgres as db;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

fn generate_nanoid() -> String {
    nanoid::nanoid!()
}

#[derive(Clone)]
pub(crate) struct AppState {
    db: db::postgres::PgPool,
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

#[tracing::instrument(skip_all, fields(%new_task.job_name))]
async fn post_tasks(
    Extension(enqueue_tasks): Extension<Arc<EnqueuedTasks>>,
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

    tracing::info!("enqueued");

    enqueue_tasks
        .tx
        .send_async(1usize)
        .await
        .expect("send task");

    Json(id)
}

pub struct Server {
    pub pool: PgPool,
    pub host: String,
    pub port: u16,
    pub on_bind: Box<dyn FnOnce() + Send + Sync>,
}

impl Server {
    pub async fn serve(self) {
        let mut api = OpenApi {
            info: Info {
                description: Some("pointguard api".to_string()),
                ..Info::default()
            },
            ..OpenApi::default()
        };

        let enqueue_tasks = EnqueuedTasks::from(flume::unbounded());

        let mut app = ApiRouter::new()
            .route("/api", Redoc::new("/api/openapi.json").axum_route())
            .route("/api/openapi.json", get(serve_api))
            .nest("/", admin_routes())
            .api_route("/api/v1/version", get(stub))
            .api_route("/api/v1/tasks", post(post_tasks))
            .api_route("/api/v1/tasks/:id/cancel", post(cancel_task))
            .api_route("/api/v1/tasks/enqueued", get(get_enqueued_tasks))
            .api_route("/api/v1/tasks/finished", get(get_finished_tasks))
            .with_state(AppState { db: self.pool })
            .finish_api_with(&mut api, |api| api.default_response::<String>())
            .layer(Extension(api))
            .layer(Extension(Arc::new(enqueue_tasks)));

        #[cfg(debug_assertions)]
        {
            let reloader = tower_livereload::LiveReloadLayer::new();
            app = app.layer(reloader);
        };

        let host = self.host;
        let port = self.port;

        let server = axum::Server::bind(&format!("{host}:{port}").parse().unwrap());
        (self.on_bind)();
        server.serve(app.into_make_service()).await.unwrap();
    }
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

async fn get_finished_tasks(State(state): State<AppState>) -> impl IntoApiResponse {
    let finished_tasks = db::finished_tasks(&state.db).await.expect("finished tasks");
    Json(finished_tasks)
}

async fn get_enqueued_tasks(State(state): State<AppState>) -> impl IntoApiResponse {
    let enqueued_tasks = db::enqueued_tasks(&state.db).await.expect("enqueued tasks");
    Json(enqueued_tasks)
}
