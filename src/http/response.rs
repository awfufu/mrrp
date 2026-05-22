use axum::{
    body::Body,
    http::{HeaderValue, StatusCode, header},
    response::Response,
};

pub fn text(body: String) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/plain; charset=utf-8"),
        )
        .body(Body::from(body))
        .expect("failed to build text response")
}

pub fn empty(status: StatusCode) -> Response {
    Response::builder()
        .status(status)
        .body(Body::empty())
        .expect("failed to build empty response")
}
