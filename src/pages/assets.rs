use axum::extract::Path;
use axum::{
    response::{Response, IntoResponse},
    http::StatusCode,
    body::{boxed, Empty, Full},
};
use include_dir::{include_dir, Dir};

static STATIC_DIR: Dir<'_> = include_dir!("static");

#[allow(clippy::unused_async)]
pub(crate) async fn static_path(Path(path): axum::extract::Path<String>) -> impl IntoResponse {
    let path = path.trim_start_matches('/');
    let mime_type = mime_guess::from_path(path).first_or_text_plain();

    match STATIC_DIR.get_file(path) {
        Some(file) => Response::builder()
            .status(StatusCode::OK)
            .header(
                axum::http::header::CONTENT_TYPE,
                axum::http::HeaderValue::from_str(mime_type.as_ref()).unwrap(),
            )
            .body(boxed(Full::from(file.contents())))
            .expect("unable to serve static content"),
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(boxed(Empty::new()))
            .expect("unable to build 404 body")
    }
}
