#[cfg(debug_assertions)]
pub(crate) use dev::serve;

#[cfg(not(debug_assertions))]
pub(crate) use release::serve;

#[cfg(debug_assertions)]
mod dev {
    use std::path::Path;

    use tower_http::services::ServeDir;

    pub fn serve() -> ServeDir {
        let path = Path::new(file!())
            .parent()
            .and_then(Path::parent)
            .and_then(Path::parent)
            .unwrap()
            .join("public");
        tower_http::services::ServeDir::new(path)
    }
}

#[cfg(not(debug_assertions))]
pub(crate) mod release {
    use std::{convert::Infallible, future::Future, pin::Pin, task::Poll};

    use axum::response::IntoResponse;
    use http::header;
    use rust_embed::RustEmbed;
    use tower::Service;

    #[derive(RustEmbed)]
    #[folder = "public"]
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
            let data = Public::get(pathname);

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
}
