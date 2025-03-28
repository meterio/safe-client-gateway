use crate::config::default_request_timeout;
#[cfg(not(test))]
use crate::config::internal_client_connect_timeout;
use crate::utils::errors::{ApiError, ApiResult};
use core::time::Duration;
use mockall::automock;
use reqwest::header::CONTENT_TYPE;

#[derive(PartialEq, Debug)]
pub struct Request {
    url: String,
    body: Option<String>,
    timeout: Duration,
}

impl Request {
    pub fn new(url: String) -> Self {
        Request {
            url,
            body: None,
            timeout: Duration::from_millis(default_request_timeout()),
        }
    }

    pub fn timeout(&mut self, timeout: Duration) -> &mut Self {
        self.timeout = timeout;
        self
    }

    pub fn body(&mut self, body: Option<String>) -> &mut Self {
        self.body = body;
        self
    }
}

#[derive(PartialEq, Debug)]
pub struct Response {
    pub body: String,
    pub status_code: u16,
}

impl Response {
    pub fn is_server_error(&self) -> bool {
        500 <= self.status_code && self.status_code < 600
    }
    pub fn is_client_error(&self) -> bool {
        400 <= self.status_code && self.status_code < 500
    }
    pub fn is_success(&self) -> bool {
        200 <= self.status_code && self.status_code < 300
    }

    /// Maps a [reqwest::Response] into a [ApiResult<Response>]
    /// If the response is a client error [400, 500[ or a server error [500, 600[ then
    /// an [ApiError] is returned as a failure. [Response] is returned otherwise.
    ///
    /// # Arguments
    ///
    /// * `reqwest_response`: The [reqwest::Response] to be mapped
    ///
    /// returns: Result<Response, ApiError>
    ///
    async fn from(reqwest_response: reqwest::Response) -> ApiResult<Self> {
        let status_code = reqwest_response.status().as_u16();
        let body: String = reqwest_response.text().await?;
        let response = Response { body, status_code };

        if response.is_client_error() || response.is_server_error() {
            Err(ApiError::from_http_response(&response))
        } else {
            Ok(response)
        }
    }
}

#[automock]
#[rocket::async_trait]
pub trait HttpClient: Send + Sync {
    async fn get(&self, request: Request) -> ApiResult<Response>;
    async fn post(&self, request: Request) -> ApiResult<Response>;
    async fn delete(&self, request: Request) -> ApiResult<Response>;
}

#[rocket::async_trait]
impl HttpClient for reqwest::Client {
    async fn get(&self, request: Request) -> ApiResult<Response> {
        let response = self
            .get(&request.url)
            .timeout(request.timeout)
            .send()
            .await?;
        Response::from(response).await
    }

    async fn post(&self, request: Request) -> ApiResult<Response> {
        let body = request.body.unwrap_or(String::from(""));
        let response = self
            .post(&request.url)
            .header(CONTENT_TYPE, "application/json")
            .body(body)
            .timeout(request.timeout)
            .send()
            .await?;
        Response::from(response).await
    }

    async fn delete(&self, request: Request) -> ApiResult<Response> {
        let body = request.body.unwrap_or(String::from(""));
        let response = self
            .delete(&request.url)
            .header(CONTENT_TYPE, "application/json")
            .body(body)
            .timeout(request.timeout)
            .send()
            .await?;
        Response::from(response).await
    }
}

#[cfg(test)]
pub fn setup_http_client() -> impl HttpClient {
    MockHttpClient::new()
}

#[cfg(not(test))]
pub fn setup_http_client() -> impl HttpClient {
    reqwest::Client::builder()
        .connect_timeout(Duration::from_millis(internal_client_connect_timeout()))
        .build()
        .unwrap()
}
