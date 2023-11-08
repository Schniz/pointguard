use aide::{
    axum::{routing::get, ApiRouter, IntoApiResponse},
    openapi::{Info, OpenApi},
    redoc::Redoc,
};
use axum::{Extension, Json};

fn v1_router() -> ApiRouter {
    ApiRouter::new().api_route("/hello", get(hello_world))
}

#[tokio::main]
async fn main() {
    let api_router = ApiRouter::new()
        .nest("/v1", v1_router())
        .route("/", Redoc::new("/api/openapi.json").axum_route())
        .route("/openapi.json", get(serve_api));

    let app = ApiRouter::new().nest("/api", api_router);

    let mut api = OpenApi {
        info: Info {
            description: Some("an example API".to_string()),
            ..Info::default()
        },
        ..OpenApi::default()
    };

    axum::Server::bind(&"127.0.0.1:8080".parse().unwrap())
        .serve(
            app.finish_api(&mut api)
                .layer(Extension(api))
                .into_make_service(),
        )
        .await
        .unwrap();
}

async fn hello_world() -> impl IntoApiResponse {
    "Hello, world!"
}

async fn serve_api(Extension(api): Extension<OpenApi>) -> impl IntoApiResponse {
    Json(api)
}
