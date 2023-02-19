use futures::future::{self, Ready};
use hyper::body::Body;
use hyper::http::{Request, Response, StatusCode};
use hyper::server::conn::AddrStream;
use hyper::service::Service;
use moss_host_call::http_impl::http_handler::{Request as HostRequest, Response as HostResponse};
use moss_runtime::pool;
use std::convert::Infallible;
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::time::Instant;
use tracing::{error, error_span, info, info_span};

struct HttpService {
    req_id: Arc<AtomicU64>,
    worker_pool: Arc<pool::WorkerPool>,
}

impl HttpService {
    fn new(wasm_file: &str, is_wasi: bool) -> Self {
        Self {
            req_id: Arc::new(AtomicU64::new(0)),
            worker_pool: Arc::new(pool::create(wasm_file, is_wasi).unwrap()),
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
        future::ok(HttpRequestContext::new(
            self.req_id.clone(),
            self.worker_pool.clone(),
        ))
    }
}

struct HttpRequestContext {
    req_id: Arc<AtomicU64>,
    worker_pool: Arc<pool::WorkerPool>,
}

impl HttpRequestContext {
    fn new(req_id: Arc<AtomicU64>, worker_pool: Arc<pool::WorkerPool>) -> Self {
        Self {
            req_id,
            worker_pool,
        }
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
        let worker_pool = self.worker_pool.clone();
        let fut = async move {
            let start_time = Instant::now();
            let mut worker = match worker_pool.get().await {
                Ok(w) => w,
                Err(e) => {
                    error_span!(
                        "[Req]",
                        req_id = req_id,
                        method = req.method().as_str(),
                        uri = req.uri().path()
                    )
                    .in_scope(|| {
                        error!("get worker failed: {}", e);
                    });
                    error!(elapsed = ?start_time.elapsed(), "get worker failed: {}", e);
                    return Ok(create_error_response(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("get worker failed: {e}"),
                    ));
                }
            };

            // convert hyper request to host-call request
            let mut headers: Vec<(&str, &str)> = vec![];
            let req_headers = req.headers().clone();
            req_headers.iter().for_each(|(k, v)| {
                headers.push((k.as_str(), v.to_str().unwrap()));
            });

            let url = req.uri().to_string();
            let method = req.method().clone();
            let body_bytes = hyper::body::to_bytes(req.body_mut()).await?.to_vec();

            let host_req = HostRequest {
                method: method.as_str(),
                uri: url.as_str(),
                headers: &headers,
                body: Some(&body_bytes),
            };

            // call worker execute
            let host_resp: HostResponse = match worker.handle_request(host_req).await {
                Ok(r) => r,
                Err(e) => {
                    error_span!(
                        "[Req]",
                        req_id = req_id,
                        method = method.as_str(),
                        uri = url.as_str()
                    )
                    .in_scope(|| {
                        error!(elapsed = ?start_time.elapsed(),"execute failed: {e}");
                    });
                    error!("execute failed: {e}");
                    return Ok(create_error_response(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("execute failed: {e}"),
                    ));
                }
            };

            // convert wasm response to hyper response
            let mut builder = Response::builder().status(host_resp.status);
            for (k, v) in host_resp.headers {
                builder = builder.header(k, v);
            }
            let resp = builder.body(Body::from(host_resp.body.unwrap())).unwrap();

            info_span!(
                "[Req]",
                req_id = req_id,
                method = method.as_str(),
                uri = url.as_str(),
                status = resp.status().as_u16()
            )
            .in_scope(|| {
                info!(elapsed = ?start_time.elapsed(),  "request finished");
            });

            Ok(resp)
        };

        Box::pin(fut)
    }
}

fn create_error_response(status: StatusCode, message: String) -> Response<Body> {
    Response::builder()
        .status(status)
        .body(Body::from(message))
        .unwrap()
}

pub async fn start(addr: SocketAddr, wasm_file: &str, is_wasi: bool) {
    let svc = HttpService::new(wasm_file, is_wasi);

    let server = match hyper::Server::try_bind(&addr) {
        Ok(server) => server.serve(svc),
        Err(e) => {
            error!("starting failed to bind: {e}");
            return;
        }
    };

    info!("starting on {}", addr);

    // Run this server for... forever!
    if let Err(e) = server.await {
        error!("starting error: {e}");
    }
}
