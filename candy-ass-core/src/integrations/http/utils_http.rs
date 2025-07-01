use crate::integrations::http::HttpResponseError;
use futures::future::BoxFuture;
use futures::{FutureExt, TryFutureExt};
use reqwest::header::HeaderMap;
use reqwest::{Error, Response, Url};
use serde::de::DeserializeOwned;

/// HttpFutureExt
pub trait HttpFutureExt: Future<Output = Result<Response, Error>> + Send {
    fn parse_json_or_error<T: DeserializeOwned + Send + 'static>(self) -> BoxFuture<'static, Result<(T, HeaderMap), HttpResponseError>>;
}
impl<F> HttpFutureExt for F
where
    F: Future<Output = Result<Response, Error>> + Send + 'static,
{
    fn parse_json_or_error<T: DeserializeOwned + Send + 'static>(self) -> BoxFuture<'static, Result<(T, HeaderMap), HttpResponseError>> {
        self.map_err(HttpResponseError::Transport)
            .and_then(|response: Response| {
                let status = response.status();
                let url = response.url().clone();
                let headers = response.headers().clone(); // клонируем заранее

                async move {
                    if status.is_success() {
                        match response.json::<T>().await {
                            Ok(json) => Ok((json, headers)),
                            Err(err) => Err(HttpResponseError::UnexpectedContent { url, source: err }),
                        }
                    } else {
                        let body = response.text().await.unwrap_or_else(|_| "<failed to read body>".into());
                        Err(HttpResponseError::UnexpectedStatus { status, url, body })
                    }
                }
            })
            .boxed()
    }
}

/// UrlBuilder
pub struct UrlBuilder {
    url: Url,
}
impl UrlBuilder {
    pub fn new(base: &str, path: &str) -> Self {
        let url = Url::parse(base).expect("Invalid base URL").join(path).expect("Invalid path URL");
        Self { url }
    }

    pub fn with_param<T: ToString>(mut self, key: &str, value: T) -> Self {
        self.url.query_pairs_mut().append_pair(key, &value.to_string());
        self
    }

    pub fn with_optional_param<T: ToString>(mut self, key: &str, value: Option<&T>) -> Self {
        if let Some(v) = value {
            self.url.query_pairs_mut().append_pair(key, &v.to_string());
        }
        self
    }

    pub fn build(self) -> Url {
        self.url
    }
}

#[cfg(test)]
mod tests {
    use crate::integrations::http::HttpResponseError;
    use crate::integrations::http::utils_http::{HttpFutureExt, UrlBuilder};
    use axum::response::IntoResponse;
    use axum::routing::get;
    use axum::{Json, Router};
    use reqwest::{Client, StatusCode};
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use std::time::Duration;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestDomain {
        message: String,
    }

    async fn ok() -> Json<serde_json::Value> {
        Json(json!({
            "message": "Hello, World!"
        }))
    }

    async fn invalid_json() -> impl IntoResponse {
        (StatusCode::OK, [("Content-Type", "lib/json")], "This is not JSON: {oops")
    }

    async fn bad_request() -> impl IntoResponse {
        let body = json!({
            "error": "Bad request",
            "code": 400
        });
        (StatusCode::BAD_REQUEST, Json(body))
    }

    async fn start_server() -> std::io::Result<()> {
        let app = Router::new()
            .route("/ok", get(ok))
            .route("/invalid", get(invalid_json))
            .route("/bad_request", get(bad_request));

        let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
        axum::serve(listener, app).await
    }

    #[tokio::test]
    async fn test_ok_response() {
        // Given
        tokio::spawn(start_server());
        tokio::time::sleep(Duration::from_millis(100)).await;

        let client = Client::new();

        // When
        let (response_200, _) = client.get("http://127.0.0.1:3000/ok").send().parse_json_or_error::<TestDomain>().await.unwrap();

        let response_400: HttpResponseError = client
            .get("http://127.0.0.1:3000/bad_request")
            .send()
            .parse_json_or_error::<TestDomain>()
            .await
            .unwrap_err();

        let response_error: HttpResponseError = client
            .get("http://127.0.0.1:3000/invalid")
            .send()
            .parse_json_or_error::<TestDomain>()
            .await
            .unwrap_err();

        let transport_error: HttpResponseError = client.get("http://?*&%$#@!").send().await.unwrap_err().into();

        // Then
        assert_eq!("Hello, World!", response_200.message);
        assert!(matches!(response_400, HttpResponseError::UnexpectedStatus { .. }));
        assert!(matches!(response_error, HttpResponseError::UnexpectedContent { .. }));
        assert!(matches!(transport_error, HttpResponseError::Transport { .. }));
    }

    #[tokio::test]
    async fn test_build_query() {
        // Given
        let builder = UrlBuilder::new("http://localhost", "/api")
            .with_param("str", "abc")
            .with_param("num", 1)
            .with_optional_param("opt_s", Some("zxc").as_ref())
            .with_optional_param("opt_n", Some(2).as_ref())
            .with_optional_param("none", None::<i32>.as_ref());

        // When
        let url = builder.build();

        // Then
        assert_eq!("http://localhost/api?str=abc&num=1&opt_s=zxc&opt_n=2", url.as_str())
    }
}
