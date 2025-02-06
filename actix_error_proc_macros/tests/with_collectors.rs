use actix_error_proc_macros::{proof_route, ActixError};
use actix_web::{web::Json, HttpResponse};
use reqwest::{Client, StatusCode};
use tokio::test;
use serde::Deserialize;
use thiserror::Error;
use crate::shared::HttpResult;

mod shared;

#[derive(ActixError, Error, Debug)]
pub enum TestError {
    #[http_status(ImATeapot)]
    #[error("test_collect")]
    Collect
}

#[derive(Deserialize)]
pub struct User {
    #[allow(unused)]
    name: String,
    #[allow(unused)]
    age: i32
}

#[proof_route(post("/"))]
#[allow(unused_variables)]
async fn test_route(#[or(TestError::Collect)] user: Json<User>) -> HttpResult<TestError> {
    Ok(HttpResponse::Ok().finish())
}

#[proof_route(post("/"))]
#[allow(unused_variables)]
async fn test2_route(user: Json<User>) -> HttpResult<TestError> {
    Ok(HttpResponse::Ok().finish())
}

#[test]
async fn should_override_to_im_a_teapot() {
    let (thread, server, address) = web_server!(test_route);

    let result = Client::new()
        .post(address)
        .body("invalid json")
        .send()
        .await
        .expect("Error while making the request.");

    assert_eq!(result.status(), StatusCode::IM_A_TEAPOT);

    let text = result
        .text()
        .await
        .expect("Error while reading response body.");

    assert_eq!(text, "test_collect");

    server.stop(true).await;
    thread.join().unwrap();
}

#[test]
async fn should_not_override() {
    let (thread, server, address) = web_server!(test2_route);

    let result = Client::new()
        .post(address)
        .body("invalid json")
        .send()
        .await
        .expect("Error while making the request.");

    assert_eq!(result.status(), StatusCode::BAD_REQUEST);

    server.stop(true).await;
    thread.join().unwrap();
}
