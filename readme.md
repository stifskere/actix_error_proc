# actix_error_proc
`actix_error_proc` is a small library to integrate `thiserror` into `actix_web` routes
with procedural macros.

This library has two main macros as well as a `thiserror` re export under the `thiserror` feature.

## `ActixError`
This macro is used together with `thiserror::Error` and it allows the user
to add a few more attributes to the error enumerable.

A basic usage of this macro looks like this

```rust
use actix_error_proc::{ActixError, Error}; // Error is a thiserror re export.

#[derive(ActixError, Error, Debug)]
enum SomeError {
    #[error("Couldn't parse http body.")]
    #[http_status(BadRequest)]
    InvalidBody,

    // if no attribute is set the default is InternalServerError.
    #[error("A database error occurred: {0:#}")]
    DatabaseError(#[from] /* ... */)
}
```

By default the response is simply the status code and the `#[error("...")]` format
as a body. But you can change that with the `transformer`.

There is another attribute you can add called `actix_error` at the enumerable level
that lets you change how the response will look, for now it only has the `transformer`
variable, but in a future it might have more things.

An example usage of the `transformer` variable looks like this

```rust
use actix_error_proc::{ActixError, Error}; // Error is a thiserror re export.

// This should not throw any error, the errors should be handled
// at the request level.
fn transform_error(mut res: HttpResponseBuilder, fmt: String) -> HttpResponse {
    res
	.insert_header(("Test", "This is a test header"))
        .json(json!({"error": fmt})) // by default the response is the raw string.
}

#[derive(ActixError, Error, Debug)]
#[actix_error(transformer = "transform_error")] // reference `transform_error` here.
enum SomeError {
	// ...
}
```

All of this is to be used with the `proof_route` attribute.

## `proof_route`

This attribute wraps an `actix_web` route changing it's result into a `Result<HttpResponse, E: Into<HttpResponse>>`
where E is your custom enumerable that implements `Into<HttpResponse>` because of the `ActixError` derive macro.

There is a type alias for that Result which is `actix_error_proc::HttpResult<E>`.

An example usage of the `proof_route` procedural macro look like this

```rust
use actix_error_proc::{ActixError, Error, HttpResult}; // Error is a thiserror re export.
use crate::models::user::User;
use actix_web::{main, App, HttpServer}
use serde_json::{from_slice, Error as JsonError};
use std::io::Error as IoError;

// assuming we have a wrapped enum
#[derive(ActixError, Error, Debug)]
enum SomeError {
    #[error("The body could not be parsed.")]
    #[status_code(BadRequest)]
    InvalidBody(#[from] JsonError)
}

#[proof_route(post("/users"))]
async fn some_route(body: Bytes) -> HttpResult<SomeError> {
    let user: User = from_slice(body)?; // notice the use of `?`.

    // Do something with the user.

    Ok(HttpResponse::NoContent()) // return Ok if everything went fine.
}

async fn main() -> IoError {
    HttpServer::new(|| {
        App::new()
            .service(some_route) // we can register the route normally.
    })
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}
```

There is an extra attribute we can add to route collectors to override
it's error status code, in the case we don't want the original status code
or we didn't create the collector and the original error does not match our
expectations we can use `#[or]`, which lets us specify an error branch
from any type instance that implements `Into<HttpResponse>`.

```rust
#[proof_route(post("/"))]
async fn route(#[or(SomeError::InvalidUser)] user: Json<User>) // ...
```

In this case if `Json<User>` fails while collecting from the http request
whatever `<SomeError::InvalidUser as Into<actix_web::HttpResponse>>.into()` returns
will be passed directly as a response for the route.

If you don't add the attribute, the request will be collected as normal and in the
case of any error the original error implementation for that collector will
be applied.
## Contributing

Before making a blind pull request please, open an issue we can talk about it and
then if it's necessary you can make a pull request, I actively maintain this project
so I'll read all the issues when possible.

You can also email me at `esteve@memw.es`. Thanks.
