use futures::future::{self, Ready};
use hyper::body::Body;
use hyper::http::{Request, Response, StatusCode};
use hyper::server::conn::AddrStream;
use hyper::service::Service;
use std::convert::Infallible;
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};
use tracing::{error, info};
struct HttpService {
    req_id: Arc<AtomicU64>,
}

impl HttpService {
    fn new(wasm_file: &str) -> Self {
        Self {
            req_id: Arc::new(AtomicU64::new(0)),
        }
    }
}

impl<'addr> Service<&'addr AddrStream> for HttpService {
    type Response = HttpRequestContext;
    type Error = Infallible;
    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _addr: &'addr AddrStream) -> Self::Future {
        future::ok(HttpRequestContext::new(self.req_id.clone()))
    }
}

struct HttpRequestContext {
    req_id: Arc<AtomicU64>,
}

impl HttpRequestContext {
    fn new(req_id: Arc<AtomicU64>) -> Self {
        Self { req_id }
    }
}

impl Service<Request<Body>> for HttpRequestContext {
    type Response = Response<Body>;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
    fn call(&mut self, mut req: Request<Body>) -> Self::Future {
        let req_id = self.req_id.fetch_add(1, Ordering::SeqCst);
        let resp = create_error_response(StatusCode::OK, "hello moss-wasm".to_string());
        Box::pin(async move { Ok(resp) })
    }
}

fn create_error_response(status: StatusCode, message: String) -> Response<Body> {
    Response::builder()
        .status(status)
        .body(Body::from(message))
        .unwrap()
}

pub async fn start(addr: SocketAddr, wasm_file: &str) {
    let svc = HttpService::new(wasm_file);

    let server = match hyper::Server::try_bind(&addr) {
        Ok(server) => server.serve(svc),
        Err(e) => {
            error!("starting failed to bind: {}", e);
            return;
        }
    };

    info!("starting on {}", addr);

    // Run this server for... forever!
    if let Err(e) = server.await {
        error!("starting error: {}", e);
    }
}
