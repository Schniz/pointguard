use aide::{
    axum::{routing::get, ApiRouter, IntoApiResponse},
    openapi::{Info, OpenApi},
    redoc::Redoc,
};
use axum::{Extension, Json};

#[tokio::main]
async fn main() {
    let app = ApiRouter::new()
        .api_route("/hello", get(hello_world))
        .route("/redoc", Redoc::new("/api.json").axum_route())
        .route("/api.json", get(serve_api));

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
