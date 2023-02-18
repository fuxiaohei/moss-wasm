wasmtime::component::bindgen!({
    world:"http-fetch",
    path: "../../wit/http-fetch.wit",
    async: true,
});

use http_fetch::{FetchError, FetchOptions, RedirectPolicy, Request, Response};
use reqwest::redirect;
use std::str::FromStr;

impl Default for FetchOptions {
    fn default() -> Self {
        FetchOptions {
            timeout: 30,
            redirect: RedirectPolicy::Follow,
        }
    }
}

pub struct FetchImpl {
    pub req_id: u64,
    pub counter: u16,
}

impl FetchImpl {
    pub fn new(req_id: u64) -> Self {
        FetchImpl { req_id, counter: 0 }
    }
}

impl TryFrom<http_fetch::RedirectPolicy> for redirect::Policy {
    type Error = anyhow::Error;
    fn try_from(value: http_fetch::RedirectPolicy) -> Result<Self, Self::Error> {
        match value {
            http_fetch::RedirectPolicy::Follow => Ok(redirect::Policy::default()),
            http_fetch::RedirectPolicy::Error => Ok(redirect::Policy::custom(|attempt| {
                attempt.error(anyhow::anyhow!("redirect policy is error"))
            })),
            http_fetch::RedirectPolicy::Manual => Ok(redirect::Policy::none()),
        }
    }
}

#[async_trait::async_trait]
impl http_fetch::HttpFetch for FetchImpl {
    async fn fetch(
        &mut self,
        request: Request,
        options: FetchOptions,
    ) -> anyhow::Result<std::result::Result<Response, FetchError>> {
        println!("request: {} {}", request.method, request.uri);

        self.counter += 1;

        let fetch_body = match request.body {
            Some(b) => b,
            None => vec![],
        };

        let client = reqwest::Client::builder()
            .redirect(options.redirect.try_into()?)
            .build()?;
        let fetch_response = match client
            .request(
                reqwest::Method::from_str(request.method.as_str()).unwrap(),
                request.uri.clone(),
            )
            .timeout(std::time::Duration::from_secs(options.timeout as u64))
            .body(reqwest::Body::from(fetch_body))
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                println!("request failed: {}", e,);
                return Ok(Err(FetchError::InvalidRequest));
            }
        };

        let mut resp_headers = vec![];
        for (key, value) in fetch_response.headers() {
            resp_headers.push((key.to_string(), value.to_str().unwrap().to_string()));
        }
        let resp = Response {
            status: fetch_response.status().as_u16(),
            headers: resp_headers,
            body: Some(fetch_response.bytes().await?.to_vec()),
        };
        println!("[Fetch] response: {}, id={}", resp.status, self.req_id,);
        Ok(Ok(resp))
    }
}
