mod fetch_impl;

pub mod http {
    use bytes::Bytes;

    pub type Request = http::Request<Bytes>;
    pub type Response = http::Response<Bytes>;

    pub fn error_response(status: http::StatusCode, message: String) -> Response {
        let mut response = Response::new(message.into());
        *response.status_mut() = status;
        response
    }

    pub use super::fetch_impl::fetch;
    pub use super::fetch_impl::FetchOptions;
    pub use super::fetch_impl::RedirectPolicy;
    pub type Error = super::fetch_impl::FetchError;
}

/// Re-export macro from moss-sdk-macro
pub use moss_sdk_macro::http_main;

mod kv_impl;

/// Re-export kv_impl as kv
pub mod kv {
    pub use crate::kv_impl::*;
}
