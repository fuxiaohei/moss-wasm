use bytes::Bytes;
use moss_sdk::http::{fetch, FetchOptions, Request, Response};
use moss_sdk::http_main;

#[http_main]
pub fn handle_sdk_http(mut _req: Request) -> Response {
    let fetch_request = http::Request::builder()
        .method("GET")
        .uri("https://www.rust-lang.org/")
        .body(Bytes::new())
        .unwrap();

    let fetch_response = fetch(fetch_request, FetchOptions::default()).unwrap();

    http::Response::builder()
        .status(fetch_response.status())
        .body(fetch_response.body().clone())
        .unwrap()
}
