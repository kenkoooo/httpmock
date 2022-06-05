use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use hyper::{Body, Request};
use serde::{Deserialize, Serialize};

use crate::common::data::{ActiveMock, ClosestMatch, MockDefinition, MockRef, RequestRequirements};
use crate::server::web::handlers::{
    add_new_mock, delete_all_mocks, delete_history, delete_one_mock, read_one_mock, verify,
};

pub mod local;
pub mod standalone;

/// Type alias for [regex::Regex](../regex/struct.Regex.html).
pub type Regex = regex::Regex;

#[derive(Debug)]
struct InternalHttpClient {
    #[cfg(feature = "hyper-client")]
    client: hyper::Client<hyper::client::HttpConnector>,

    #[cfg(feature = "isahc-client")]
    client: isahc::HttpClient,
}
impl InternalHttpClient {
    #[cfg(feature = "hyper-client")]
    fn new() -> Arc<Self> {
        let mut connector = hyper::client::HttpConnector::new();
        connector.set_keepalive(Some(Duration::from_secs(60 * 60 * 24)));
        let client = hyper::Client::builder().build(connector);
        Arc::new(Self { client })
    }

    #[cfg(feature = "isahc-client")]
    fn new() -> Arc<Self> {
        use isahc::config::Configurable;
        let client = isahc::HttpClient::builder()
            .tcp_keepalive(Duration::from_secs(60 * 60 * 24))
            .build()
            .expect("Cannot build HTTP client");
        Arc::new(Self { client })
    }

    #[cfg(feature = "hyper-client")]
    async fn execute_request(&self, req: Request<Body>) -> Result<(u16, String), String> {
        let response = self
            .client
            .request(req)
            .await
            .map_err(|err| format!("cannot send request to mock server: {}", err))?;
        let status = response.status();
        let body = hyper::body::to_bytes(response.into_body())
            .await
            .map_err(|err| format!("cannot send request to mock server: {}", err))?;
        let body = String::from_utf8(body.to_vec())
            .map_err(|err| format!("cannot send request to mock server: {}", err))?;

        Ok((status.as_u16(), body))
    }

    #[cfg(feature = "isahc-client")]
    async fn execute_request(&self, req: Request<Body>) -> Result<(u16, String), String> {
        use isahc::AsyncReadResponseExt;

        let (parts, body) = req.into_parts();
        let body = hyper::body::to_bytes(body)
            .await
            .map_err(|err| format!("cannot load request body: {}", err))?
            .to_vec();
        let req = Request::from_parts(parts, body);
        let mut response = self
            .client
            .send_async(req)
            .await
            .map_err(|err| format!("cannot send request to mock server: {}", err))?;
        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|err| format!("cannot send request to mock server: {}", err))?;
        Ok((status.as_u16(), body))
    }
    async fn http_ping(&self, server_addr: &SocketAddr) -> Result<(), String> {
        let request_url = format!("http://{}/__httpmock__/ping", server_addr);
        let request = Request::builder()
            .method(hyper::Method::GET)
            .uri(request_url)
            .body(Body::from(""))
            .unwrap();

        let (status, _body) = match self.execute_request(request).await {
            Err(err) => return Err(format!("cannot send request to mock server: {}", err)),
            Ok(sb) => sb,
        };

        if status != 200 {
            return Err(format!(
                "Could not create mock. Mock server response: status = {}",
                status
            ));
        }

        Ok(())
    }
}

/// Represents an HTTP method.
#[derive(Serialize, Deserialize, Debug)]
pub enum Method {
    GET,
    HEAD,
    POST,
    PUT,
    DELETE,
    CONNECT,
    OPTIONS,
    TRACE,
    PATCH,
}

impl FromStr for Method {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "GET" => Ok(Method::GET),
            "HEAD" => Ok(Method::HEAD),
            "POST" => Ok(Method::POST),
            "PUT" => Ok(Method::PUT),
            "DELETE" => Ok(Method::DELETE),
            "CONNECT" => Ok(Method::CONNECT),
            "OPTIONS" => Ok(Method::OPTIONS),
            "TRACE" => Ok(Method::TRACE),
            "PATCH" => Ok(Method::PATCH),
            _ => Err(format!("Invalid HTTP method {}", input)),
        }
    }
}

impl From<&str> for Method {
    fn from(value: &str) -> Self {
        value.parse().expect("Cannot parse HTTP method")
    }
}

impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

#[async_trait]
pub trait MockServerAdapter {
    fn host(&self) -> String;
    fn port(&self) -> u16;
    fn address(&self) -> &SocketAddr;
    async fn create_mock(&self, mock: &MockDefinition) -> Result<MockRef, String>;
    async fn fetch_mock(&self, mock_id: usize) -> Result<ActiveMock, String>;
    async fn delete_mock(&self, mock_id: usize) -> Result<(), String>;
    async fn delete_all_mocks(&self) -> Result<(), String>;
    async fn verify(&self, rr: &RequestRequirements) -> Result<Option<ClosestMatch>, String>;
    async fn delete_history(&self) -> Result<(), String>;
    async fn ping(&self) -> Result<(), String>;
}
