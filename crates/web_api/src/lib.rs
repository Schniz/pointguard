mod admin;
mod events;
mod router;

use axum::Extension;
use db::postgres::PgPool;
use events::EnqueuedTasks;
use futures::Future;
use pointguard_engine_postgres as db;
use std::sync::Arc;

pub use router::api_router;

#[derive(Clone)]
pub struct AppState {
    db: db::postgres::PgPool,
}

pub struct Server {
    pub pool: PgPool,
    pub host: String,
    pub port: u16,
    pub on_bind: Box<dyn FnOnce(&str, u16) + Send + Sync>,
}

impl Server {
    pub async fn serve(self, shutdown_signal: impl Future<Output = ()>) {
        let enqueue_tasks = EnqueuedTasks::from(flume::unbounded());

        let (app, api) = api_router();
        let mut app = app
            .with_state(AppState { db: self.pool })
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
        (self.on_bind)(&host, self.port);
        server
            .serve(app.into_make_service())
            .with_graceful_shutdown(shutdown_signal)
            .await
            .unwrap();
    }
}
