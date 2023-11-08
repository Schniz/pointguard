use aide::{
    axum::{
        routing::{get, post},
        ApiRouter, IntoApiResponse,
    },
    openapi::{Info, OpenApi},
    redoc::Redoc,
};
use axum::{extract::State, Extension, Json};
use db::postgres::PgPool;
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

/// Post task?
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
    .unwrap();
    Json(id)
}

// tracing::info_span!("task_invocation", id = %task.id, endpoint = %task.endpoint);
#[tracing::instrument(skip_all, fields(id = %task.id, endpoint = %task.endpoint))]
async fn execute_task(http: reqwest::Client, task: db::InflightTask, db: PgPool) {
    let response = http
        .post(&task.endpoint)
        .json(&task.data)
        .send()
        .await
        .and_then(|res| res.error_for_status());

    match response {
        Ok(_) => {
            tracing::info!("invocation completed");
            task.done(&db).await;
        }
        Err(err) => {
            tracing::error!("invocation failed: {err}");
            task.release(&db).await;
        }
    }
}

async fn task_queue_loop(db: db::postgres::PgPool) {
    loop {
        let mut tasks = db::free_tasks(&db, 5).await.unwrap_or_else(|err| {
            tracing::error!("Can't fetch tasks: {err}");
            vec![]
        });

        if tasks.is_empty() {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            continue;
        }

        let http = reqwest::Client::new();

        for task in tasks.drain(..) {
            tokio::spawn(execute_task(http.clone(), task, db.clone()));
        }
    }
}

#[tokio::main]
async fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "pointguard=debug");
    }

    tracing_subscriber::fmt().pretty().init();
    tracing::info!("Set up!");

    let db = db::postgres::PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    tokio::spawn(task_queue_loop(db.clone()));

    let mut api = OpenApi {
        info: Info {
            description: Some("an example API".to_string()),
            ..Info::default()
        },
        ..OpenApi::default()
    };

    let app = ApiRouter::new()
        .api_route_with("/api/v1/version", get(stub), |r| {
            r.tag("v1").description("hello").summary("what?")
        })
        .api_route_with("/api/v1/tasks", post(post_tasks), |r| {
            r.tag("v1").description("hello").summary("what?")
        })
        .route("/api", Redoc::new("/api/openapi.json").axum_route())
        .route("/api/openapi.json", get(serve_api))
        .with_state(AppState { db })
        .finish_api_with(&mut api, |api| api.default_response::<String>())
        .layer(Extension(api));

    axum::Server::bind(&"127.0.0.1:8080".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn serve_api(Extension(api): Extension<OpenApi>) -> impl IntoApiResponse {
    Json(api)
}
