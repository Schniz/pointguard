use aide::{
    axum::{
        routing::{get, post},
        ApiRouter, IntoApiResponse,
    },
    openapi::{Info, OpenApi},
    redoc::Redoc,
};
use axum::{extract::State, Extension, Json};
use pointguard_engine_postgres::{
    sea_orm::{sea_query::OnConflict, Database, DatabaseConnection},
    task,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

fn generate_nanoid() -> String {
    nanoid::nanoid!()
}

#[derive(Clone)]
struct AppState {
    db: DatabaseConnection,
}

async fn stub() -> impl IntoApiResponse {
    "Hello, world!"
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct NewTask {
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
}

/// Post task?
async fn post_tasks(
    State(state): State<AppState>,
    Json(new_task): Json<NewTask>,
) -> impl IntoApiResponse {
    use pointguard_engine_postgres::sea_orm::{prelude::*, Set};

    let task = task::Entity::insert(task::ActiveModel {
        name: Set(new_task.name.unwrap_or_else(generate_nanoid)),
        job_name: Set(new_task.job_name),
        data: Set(new_task.data.unwrap_or_default()),
        ..Default::default()
    })
    .on_conflict(
        OnConflict::columns(vec![task::Column::Name, task::Column::JobName])
            .do_nothing()
            .to_owned(),
    )
    .exec(&state.db)
    .await
    .unwrap();

    Json(task.last_insert_id)
}

#[tokio::main]
async fn main() {
    let db = Database::connect(std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    let mut api = OpenApi {
        info: Info {
            description: Some("an example API".to_string()),
            ..Info::default()
        },
        ..OpenApi::default()
    };

    let app = ApiRouter::new()
        .api_route_with("/api/v1/version", get(stub), |r| r.description("hello"))
        .api_route("/api/v1/tasks", post(post_tasks))
        .route("/api", Redoc::new("/api/openapi.json").axum_route())
        .route("/api/openapi.json", get(serve_api))
        .with_state(AppState { db })
        .finish_api(&mut api)
        .layer(Extension(api));

    axum::Server::bind(&"127.0.0.1:8080".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn serve_api(Extension(api): Extension<OpenApi>) -> impl IntoApiResponse {
    Json(api)
}
