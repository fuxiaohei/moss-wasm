use bytes::Bytes;
use moss_sdk::http::{router, Request, Response};
use moss_sdk::http_main;

#[http_main]
pub fn handle_sdk_http(mut req: Request) -> Response {
    router::get("/hello", echo_hello).unwrap();
    router::get("/foo/bar", echo_foo_bar).unwrap();
    router::route(req)
}

pub fn echo_hello(_req: Request) -> Response {
    http::Response::builder()
        .status(200)
        .body(Bytes::from("Hello, World"))
        .unwrap()
}

pub fn echo_foo_bar(_req: Request) -> Response {
    http::Response::builder()
        .status(200)
        .body(Bytes::from("Foo Bar"))
        .unwrap()
}
