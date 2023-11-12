mod admin;

use admin::{admin_routes, attach_views_reloader};
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

        let mut app = ApiRouter::new()
            .route("/api", Redoc::new("/api/openapi.json").axum_route())
            .route("/api/openapi.json", get(serve_api))
            .nest("/", admin_routes())
            .api_route_with("/api/v1/version", get(stub), |r| {
                r.description("hello").summary("what?")
            })
            .api_route_with("/api/v1/tasks", post(post_tasks), |r| {
                r.description("hello").summary("what?")
            })
            .with_state(AppState { db: self.pool })
            .finish_api_with(&mut api, |api| api.default_response::<String>())
            .layer(Extension(api));

        #[cfg(debug_assertions)]
        {
            let reloader = tower_livereload::LiveReloadLayer::new();
            let views = attach_views_reloader(reloader.reloader());
            app = app
                .layer(reloader)
                .layer(axum::Extension(std::sync::Arc::new(views)));
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
