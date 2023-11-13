use std::{convert::Infallible, future::Future, pin::Pin, task::Poll};

use axum::response::IntoResponse;
use http::header;
use rust_embed::RustEmbed;
use tower::Service;

#[derive(RustEmbed)]
#[folder = "../../packages/web-ui/dist"]
pub struct Public;

#[derive(Clone)]
pub(crate) struct ServePublic;

pub(crate) fn serve() -> ServePublic {
    ServePublic
}

impl<ReqBody> Service<http::Request<ReqBody>> for ServePublic {
    type Response = axum::response::Response;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: http::Request<ReqBody>) -> Self::Future {
        let pathname = req.uri().path();
        let pathname = pathname.strip_prefix("/").unwrap_or(pathname);
        let mut data = Public::get(pathname);

        if let None = data {
            if !pathname.starts_with("assets/") {
                data = Public::get("index.html");
            }
        }

        let resp = match data {
            None => http::status::StatusCode::NOT_FOUND.into_response(),
            Some(data) => (
                [(header::CONTENT_TYPE, data.metadata.mimetype())],
                data.data,
            )
                .into_response(),
        };

        let fut = async { Ok(resp) };

        return Box::pin(fut);
    }
}
