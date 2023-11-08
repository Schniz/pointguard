mod logging;
mod task_loop;

use aide::{
    axum::{
        routing::{get, post},
        ApiRouter, IntoApiResponse,
    },
    openapi::{Info, OpenApi},
    redoc::Redoc,
};
use axum::{extract::State, Extension, Json};
use pointguard_engine_postgres as db;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

fn generate_nanoid() -> String {
    nanoid::nanoid!()
}

#[derive(Clone)]
struct AppState {
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
}

#[tracing::instrument(skip_all, fields(%new_task.job_name))]
async fn post_tasks(
    State(state): State<AppState>,
    Json(new_task): Json<NewTaskBody>,
) -> impl IntoApiResponse {
    let id = db::enqueue(
        &state.db,
        &db::NewTask {
            job_name: new_task.job_name,
            data: new_task.data.unwrap_or_default(),
            endpoint: new_task.endpoint.to_string(),
            name: new_task.name.unwrap_or_else(generate_nanoid),
            run_at: new_task.run_at,
        },
    )
    .await
    .expect("enqueue task");
    Json(id)
}

#[tokio::main]
async fn main() {
    logging::init();
    let db = db::connect(&std::env::var("DATABASE_URL").expect("database url"))
        .await
        .expect("connect to db");

    tokio::spawn(task_loop::run(db.clone()));

    let mut api = OpenApi {
        info: Info {
            description: Some("pointguard api".to_string()),
            ..Info::default()
        },
        ..OpenApi::default()
    };

    let app = ApiRouter::new()
        .route("/api", Redoc::new("/api/openapi.json").axum_route())
        .route("/api/openapi.json", get(serve_api))
        .api_route_with("/api/v1/version", get(stub), |r| {
            r.description("hello").summary("what?")
        })
        .api_route_with("/api/v1/tasks", post(post_tasks), |r| {
            r.description("hello").summary("what?")
        })
        .with_state(AppState { db })
        .finish_api_with(&mut api, |api| api.default_response::<String>())
        .layer(Extension(api));

    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());

    let server = axum::Server::bind(&format!("{host}:{port}").parse().unwrap());
    logging::print_welcome_message(&host, &port);
    server.serve(app.into_make_service()).await.unwrap();
}

async fn serve_api(Extension(api): Extension<OpenApi>) -> impl IntoApiResponse {
    Json(api)
}
