use actix_web::HttpResponse;

pub use actix_error_proc_macros::{proof_route, ActixError};
#[cfg(feature = "thiserror")]
pub use thiserror::Error;
/// This is a type alias that you can use as http
/// route handler result, it binds to `Result<HttpResponse, E>`.
#[deprecated(since="1.2.0", note="Please consider using https://github.com/FlakySL/actix_failwrap instead.")]
pub type HttpResult<E> = Result<HttpResponse, E>;
