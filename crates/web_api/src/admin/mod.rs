mod public;

use crate::AppState;
use aide::axum::ApiRouter;
use axum::{response::Redirect, routing::get};

pub(crate) fn admin_routes() -> ApiRouter<AppState> {
    ApiRouter::new()
        .route("/", get(|| async { Redirect::to("/admin/enqueued") }))
        .nest_service("/admin", public::serve())
}
