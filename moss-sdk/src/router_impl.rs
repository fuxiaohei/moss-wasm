use crate::http::{error_response, Request, Response};
use http::{Method, StatusCode};
use matchit::Router;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::str::FromStr;
use std::sync::{Arc, Mutex};

pub trait Handler: Send + Sync + 'static {
    fn call<'a>(&'a self, req: Request) -> Response;
}

impl Debug for dyn Handler {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("dyn Handler").finish()
    }
}

impl<F> Handler for F
where
    F: Send + Sync + 'static + Fn(Request) -> Response,
{
    fn call<'a>(&'a self, req: Request) -> Response {
        (self)(req)
    }
}

// RouteHandler is router with method
type RouteHandler = HashMap<Method, Router<Arc<dyn Handler>>>;

lazy_static::lazy_static! {
     static ref ROUTER: Mutex<RouteHandler> = Mutex::new(HashMap::new());
}

macro_rules! method_route {
    ($method:ident) => {
        pub fn $method(
            path: &str,
            handler: impl Handler,
        ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            let method_name = Method::from_str(&stringify!($method).to_uppercase())?;
            ROUTER
                .lock()
                .unwrap()
                .entry(method_name)
                .or_default()
                .insert(path, Arc::new(handler))?;
            Ok(())
        }
    };
}

method_route!(post);
method_route!(get);
method_route!(put);
method_route!(delete);
method_route!(head);
method_route!(options);
method_route!(patch);

pub fn any(
    path: &str,
    handler: impl Handler + std::marker::Copy,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    get(path, handler)?;
    post(path, handler)?;
    put(path, handler)?;
    delete(path, handler)?;
    head(path, handler)?;
    options(path, handler)?;
    patch(path, handler)?;
    Ok(())
}

pub fn route(req: Request) -> Response {
    // get method and path to match router
    let method = req.method().clone();
    let path = req.uri().clone();
    let path = path.path();

    // get router
    let mut router = ROUTER.lock().unwrap();
    let router = router.entry(method).or_default();

    // match router
    let matched = router.at(path);
    if matched.is_err() {
        return error_response(StatusCode::NOT_FOUND, "route mismatch".to_string());
    }

    // call handler
    let handler = matched.unwrap().value;
    handler.call(req)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http::{Request, Response};

    #[test]
    fn test_handler_impl() {
        let handler: Arc<dyn Handler> =
            Arc::new(|_req: Request| Response::new(bytes::Bytes::from("Hello, World")));
        let req = http::Request::builder()
            .uri("/")
            .body(bytes::Bytes::from(""))
            .unwrap();
        let resp = handler.call(req);
        assert_eq!(resp.status(), 200);
        assert_eq!(resp.body(), "Hello, World");
    }

    pub fn test_route_1(req: Request) -> Response {
        let url = req.uri().clone();
        let method = req.method().to_string().to_uppercase();
        http::Response::builder()
            .status(200)
            .header("X-Request-Url", url.to_string())
            .header("X-Request-Method", method)
            .body(bytes::Bytes::from("Hello, World"))
            .unwrap()
    }

    #[test]
    fn test_router() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        {
            get("/abc", test_route_1)?;
        }

        {
            let mut router = ROUTER.lock().unwrap();
            let router = router.entry(Method::GET).or_default();
            let matched = router.at("/abcd");
            if matched.is_err() {
                assert_eq!(matched.err().unwrap(), matchit::MatchError::NotFound);
            }
        }

        {
            let mut router = ROUTER.lock().unwrap();
            let router = router.entry(Method::GET).or_default();
            let matched = router.at("/abc");
            assert!(matched.is_ok());
            let handler = matched.unwrap();
            let req = http::Request::builder()
                .method(Method::GET)
                .uri("/abc")
                .body(bytes::Bytes::from(""))
                .unwrap();
            let resp = handler.value.call(req);
            assert_eq!(resp.status(), 200);
        }
        Ok(())
    }

    #[test]
    fn test_wildcard() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        {
            any("/xyz/*path", test_route_1)?;
        }
        {
            let mut router = ROUTER.lock().unwrap();
            let router = router.entry(Method::GET).or_default();
            let matched = router.at("/xyz/abc");
            assert!(matched.is_ok());
            let path_str = matched.unwrap().params.get("path").unwrap();
            assert_eq!(path_str, "abc");
        }
        Ok(())
    }
}
