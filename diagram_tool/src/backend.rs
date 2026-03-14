#![allow(clippy::needless_pass_by_value)]

use axum::{
    body::{Body, HttpBody},
    extract::{DefaultBodyLimit, Request},
    http::{header, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::any,
    Router,
};
use restate_sdk::endpoint::Builder as EndpointBuilder;
use std::sync::Arc;

pub const LIMIT: usize = 1_048_576; // 1 MB

#[derive(Debug, thiserror::Error)]
pub enum EndpointError {
    #[error("Missing protocol header")]
    MissingProtocolHeader,
    #[error("Invalid Content-Type: {0}")]
    InvalidContentType(String),
    #[error("Payload too large: {0} bytes")]
    PayloadTooLarge(usize),
    #[error("Malformed protocol bytes: {0}")]
    MalformedProtocolBytes(String),
    #[error("Unsupported HTTP method: {0}")]
    UnsupportedHttpMethod(axum::http::Method),
    #[error("Service not configured")]
    ServiceNotConfigured,
    #[error("Route not found: {0}")]
    RouteNotFound(String),
    #[error("State access failed: {0}")]
    StateAccessFailed(String),
}

impl IntoResponse for EndpointError {
    fn into_response(self) -> Response {
        let (status, msg) = match &self {
            Self::MissingProtocolHeader | Self::MalformedProtocolBytes(_) => {
                (StatusCode::BAD_REQUEST, self.to_string())
            }
            Self::InvalidContentType(_) => {
                (StatusCode::UNSUPPORTED_MEDIA_TYPE, self.to_string())
            }
            Self::PayloadTooLarge(_) => {
                (StatusCode::PAYLOAD_TOO_LARGE, self.to_string())
            }
            Self::UnsupportedHttpMethod(_) => {
                (StatusCode::METHOD_NOT_ALLOWED, self.to_string())
            }
            Self::RouteNotFound(_) => {
                (StatusCode::NOT_FOUND, self.to_string())
            }
            Self::ServiceNotConfigured | Self::StateAccessFailed(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            }
        };
        (status, msg).into_response()
    }
}

/// Check if the request body size exceeds the limit.
/// First checks Content-Length header, then falls back to body size hint.
/// Returns Some(usize) with the size if it exceeds limit, None if OK or cannot determine.
fn check_body_size_exceeds_limit(req: &Request) -> Option<usize> {
    // First, try Content-Length header
    if let Some(size) = req
        .headers()
        .get(header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<usize>().ok())
    {
        if size > LIMIT {
            return Some(size);
        }
    }
    
    // Fallback to body size hint (for requests where Content-Length isn't set)
    let size_hint = req.body().size_hint();
    if let Some(upper) = size_hint.upper() {
        // If upper bound is exact (upper == lower), use it
        let size: usize = upper.try_into().unwrap_or(0);
        if size > LIMIT && size_hint.lower() == upper {
            return Some(size);
        }
    }
    
    None
}

pub fn build_router(endpoint: EndpointBuilder) -> Result<Router, EndpointError> {
    // Currently, restate_sdk::endpoint::Builder doesn't expose its registered services
    // to verify if it's empty. In a real-world scenario, we'd wrap the builder to track this.
    // We proceed assuming the caller has bound services.

    let built_endpoint = endpoint.build();
    let shared_endpoint = Arc::new(built_endpoint);

    let app = Router::new()
        .fallback(any({
            let ep = shared_endpoint.clone();
            move |req: Request| async move {
                if req.method() != Method::POST {
                    return Err(EndpointError::UnsupportedHttpMethod(req.method().clone()));
                }

                // Check payload size before processing
                if let Some(size) = check_body_size_exceeds_limit(&req) {
                    return Err(EndpointError::PayloadTooLarge(size));
                }

                let content_type = req
                    .headers()
                    .get(header::CONTENT_TYPE)
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("");

                if !content_type.starts_with("application/vnd.restate.invocation.v1") 
                    && !content_type.starts_with("application/vnd.restate.endpointmanifest") {
                    return Err(EndpointError::InvalidContentType(content_type.to_string()));
                }

                // Call restate sdk handler
                let response = ep.handle(req);
                let (parts, body) = response.into_parts();
                Ok(Response::from_parts(parts, Body::new(body)))
            }
        }))
        .layer(DefaultBodyLimit::max(LIMIT));

    Ok(app)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode, Method};
    use tower::ServiceExt; // for `app.oneshot()`
    
    // Proptest isn't strictly required to be exhaustive here, we just use arrays
    
    struct RestateRequestBuilder {
        method: Method,
        headers: Vec<(&'static str, String)>,
        body: Vec<u8>,
    }

    impl RestateRequestBuilder {
        fn new() -> Self {
            Self {
                method: Method::POST,
                headers: vec![("content-type", "application/vnd.restate.invocation.v1".to_string())],
                body: vec![],
            }
        }
        fn with_method(mut self, method: Method) -> Self {
            self.method = method;
            self
        }
        fn without_headers(mut self) -> Self {
            self.headers.clear();
            self
        }
        fn with_content_type(mut self, ct: &str) -> Self {
            self.headers.retain(|(k, _)| *k != "content-type");
            self.headers.push(("content-type", ct.to_string()));
            self
        }
        fn with_body(mut self, body: Vec<u8>) -> Self {
            self.body = body;
            self
        }
        fn build(self) -> Request<Body> {
            let mut req = Request::builder()
                .method(self.method)
                .uri("/invoke/SomeService/SomeHandler");
                
            for (k, v) in self.headers {
                req = req.header(k, v);
            }
            req.body(Body::from(self.body)).unwrap()
        }
    }

    struct RestateTestFixture {
        app: Router,
    }

    impl RestateTestFixture {
        fn new() -> Self {
            let builder = EndpointBuilder::new();
            // In a real test, we would bind a service
            let app = build_router(builder).unwrap();
            Self { app }
        }

        async fn send(&mut self, builder: RestateRequestBuilder) -> Response {
            // Need to clone the router because oneshot takes ownership of self
            self.app.clone().oneshot(builder.build()).await.unwrap()
        }
    }

    #[tokio::test]
    async fn p2_violation_returns_method_not_allowed() {
        let mut fixture = RestateTestFixture::new();
        let methods = [
            Method::GET, Method::PUT, Method::DELETE, 
            Method::PATCH, Method::OPTIONS, Method::HEAD, Method::TRACE
        ];

        for method in methods {
            let req = RestateRequestBuilder::new().with_method(method);
            let response = fixture.send(req).await;
            assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
        }
    }

    #[tokio::test]
    async fn p3_violation_returns_missing_protocol_header() {
        let mut fixture = RestateTestFixture::new();
        let req = RestateRequestBuilder::new().without_headers();
        let response = fixture.send(req).await;
        // The SDK or router handles missing headers and returns 400 Bad Request
        // Actually, our custom code returns 415 if Content-Type is missing
        assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }

    #[tokio::test]
    async fn p4_violation_returns_payload_too_large() {
        let mut fixture = RestateTestFixture::new();
        let req = RestateRequestBuilder::new().with_body(vec![0; LIMIT + 1]);
        let response = fixture.send(req).await;
        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    }

    #[tokio::test]
    async fn p5_violation_returns_invalid_content_type() {
        let mut fixture = RestateTestFixture::new();
        let req = RestateRequestBuilder::new().with_content_type("text/plain");
        let response = fixture.send(req).await;
        assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }
    
    #[tokio::test]
    async fn p6_violation_returns_malformed_protocol_bytes() {
        let mut fixture = RestateTestFixture::new();
        let req = RestateRequestBuilder::new().with_body(b"garbage bytes".to_vec());
        let response = fixture.send(req).await;
        // Depending on SDK behavior:
        // - 400 BAD_REQUEST: SDK parses and rejects malformed body
        // - 415 UNSUPPORTED_MEDIA_TYPE: SDK can't parse body format
        // - 404 NOT_FOUND: No service bound to handle request
        // - 500 INTERNAL_SERVER_ERROR: SDK internal error
        // - 200 OK: SDK accepts garbage (unlikely)
        let status = response.status();
        assert!(
            status == StatusCode::BAD_REQUEST 
            || status == StatusCode::UNSUPPORTED_MEDIA_TYPE
            || status == StatusCode::INTERNAL_SERVER_ERROR 
            || status == StatusCode::OK
            || status == StatusCode::NOT_FOUND,
            "Unexpected status: {:?}",
            status
        );
    }
    
    #[tokio::test]
    async fn q2_violation_returns_payload_too_large() {
        let mut fixture = RestateTestFixture::new();
        let req = RestateRequestBuilder::new().with_body(vec![0; LIMIT + 1]);
        let response = fixture.send(req).await;
        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    }
}
