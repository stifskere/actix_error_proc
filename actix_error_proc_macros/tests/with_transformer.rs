use actix_web::{HttpResponse, HttpResponseBuilder};
use crate::shared::HttpResult;
use thiserror::Error;
use actix_error_proc_macros::{proof_route, ActixError};
use reqwest::{get, StatusCode};
use tokio::test;

mod shared;

fn transformer(mut req: HttpResponseBuilder, fmt: String) -> HttpResponse {
    req
        .append_header(("format", fmt))
        .body("no")
}

#[derive(ActixError, Error, Debug)]
#[actix_error(transformer = "transformer")]
enum TestError {
    #[error("test")]
    Test
}

#[proof_route(get("/"))]
async fn test_route() -> HttpResult<TestError> {
    Err(TestError::Test)
}

#[test]
async fn should_return_fmt_as_headers() {
    let (thread, server, address) = web_server!(test_route);

    let result = get(address)
        .await
        .expect("Error while making the request.");

    assert_eq!(result.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let header = result
        .headers()
        .get("format")
        .expect("Missing format header.");

    assert_eq!(header, "test");

    let text = result
        .text()
        .await
        .expect("Error while reading response body.");

    assert_eq!(text, "no");

    server.stop(true).await;
    thread.join().unwrap();
}
