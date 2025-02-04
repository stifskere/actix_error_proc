use actix_error_proc::{proof_route, ActixError, Error, HttpResult};
use reqwest::{get, Client, StatusCode};
use tokio::test;

mod shared;

#[derive(ActixError, Error, Debug)]
enum TestError {
    #[error("test")]
    #[http_status(BadRequest)]
    Test,

    #[error("test2")]
    #[http_status(Unauthorized)]
    Test2,

    #[error("test3")]
    Test3
}

#[proof_route(get("/"))]
async fn test_route() -> HttpResult<TestError> {
    Err(TestError::Test)
}

#[proof_route(post("/"))]
async fn test2_route() -> HttpResult<TestError> {
    Err(TestError::Test2)
}

#[proof_route(get("/"))]
async fn test3_route() -> HttpResult<TestError> {
    Err(TestError::Test3)
}

#[test]
async fn should_bad_request_and_test_on_get() {
    let (thread, server, address) = web_server!(test_route);

    let result = get(address)
        .await
        .expect("Error while making the request.");

    assert_eq!(result.status(), StatusCode::BAD_REQUEST);

    let text = result
        .text()
        .await
        .expect("Error while reading response body.");

    assert_eq!(text, "test");

    server.stop(true).await;
    thread.join().unwrap();
}

#[test]
async fn should_unauthorized_and_test2_on_post() {
    let (thread, server, address) = web_server!(test2_route);

    let result = Client::new()
        .post(address)
        .send()
        .await
        .expect("Error while making the request.");

    assert_eq!(result.status(), StatusCode::UNAUTHORIZED);

    let text = result
        .text()
        .await
        .expect("Error while reading response body.");

    assert_eq!(text, "test2");

    server.stop(true).await;
    thread.join().unwrap();
}

#[test]
async fn default_is_internal_server_error() {
    let (thread, server, address) = web_server!(test3_route);

    let result = get(address)
        .await
        .expect("Error while making the request.");

    assert_eq!(result.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let text = result
        .text()
        .await
        .expect("Error while reading response body.");

    assert_eq!(text, "test3");

    server.stop(true).await;
    thread.join().unwrap();
}
