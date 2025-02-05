use core::panic;
use proc_macro::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, spanned::Spanned, Data, DeriveInput, Expr, ExprCall, ExprLit, Fields, FnArg,
    Ident, ItemFn, Lit, Meta,
};

/// This macro is helps the HttpResult type to infer
/// `thiserror::Error` errors and convert it to `actix_web::HttpResponse`
/// with attributes.
///
/// Example usage with `thiserror` could look like this:
///
/// ```ignore
/// use actix_error_proc::{ActixError, Error}; // Error is a thiserror re export.
///
/// #[derive(ActixError, Error, Debug)]
/// enum SomeError {
///     #[error("Couldn't parse http body.")]
///     #[http_status(BadRequest)]
///     InvalidBody,
///
///     // if no attribute is set the default is InternalServerError.
///     #[error("A database error occurred: {0:#}")]
///     DatabaseError(#[from] /* ... */)
/// }
/// ```
///
/// You can also add an attribute to the enum that lets you
/// modify the behaviour of how the enum is converted into an
/// `actix_web::HttpResponse`.
///
/// The only variable available for now is a transformer, which is
/// a function that transforms the request, letting you add headers
/// and other things in the response.
///
/// ```ignore
/// use actix_error_proc::{ActixError, Error}; // Error is a thiserror re export.
///
/// // This should not throw any error, the errors should be handled
/// // at the request level.
/// fn transform_error(mut res: HttpResponseBuilder, fmt: String) -> HttpResponse {
///     res
///         .insert_header(("Test", "This is a test header"))
///         .json(json!({"error": fmt})) // by default the response is the raw string.
/// }
///
/// #[derive(ActixError, Error, Debug)]
/// #[actix_error(transformer = "transform_error")] // reference `transform_error` here.
/// enum SomeError {
///  // ...
/// }
/// ```
///
/// And after that all the responses derived from the enum should have your own
/// format.
#[proc_macro_derive(ActixError, attributes(http_status, actix_error))]
pub fn derive_actix_error(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let enum_name = &input.ident;

    let Data::Enum(data_enum) = &input.data else {
        panic!("ActixError can only be derived for enums");
    };

    let transformers = input
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("actix_error"))
        .collect::<Vec<_>>();

    if transformers.len() > 1 {
        panic!("The `actix_error` attribute is exclusive, only one can exist at the same time.");
    }

    let transformer = transformers.iter().next().and_then(|attr| {
        if let Ok(Meta::NameValue(meta)) = attr.parse_args() {
            if meta.path.is_ident("transformer") {
                if let Expr::Lit(ExprLit {
                    lit: Lit::Str(lit_str),
                    ..
                }) = meta.value
                {
                    return Some(Ident::new(&lit_str.value(), Span::call_site().into()));
                }
            }
        }

        None
    });

    let mut arms = Vec::new();

    for variant in &data_enum.variants {
        let mut http_method = quote! { actix_web::HttpResponse::InternalServerError() };
        let variant_name = &variant.ident;

        for attr in &variant.attrs {
            if attr.path().is_ident("http_status") {
                if let Ok(ident) = attr.parse_args::<Ident>() {
                    http_method = quote! { actix_web::HttpResponse::#ident() };
                }
            }
        }

        let pattern = match &variant.fields {
            Fields::Unnamed(_) => quote! { Self::#variant_name(..) },
            Fields::Named(_) => quote! { Self::#variant_name { .. } },
            Fields::Unit => quote! { Self::#variant_name },
        };

        arms.push(match transformer {
            Some(ref tr) => quote! { #pattern => #tr(#http_method, format!("{:#}", self)) },
            None => quote! { #pattern => #http_method.body(format!("{:#}", self)) },
        });
    }

    let expanded = quote! {
        impl ::core::convert::Into<actix_web::HttpResponse> for #enum_name {
            fn into(self) -> actix_web::HttpResponse {
                match self {
                    #(#arms),*
                }
            }
        }
    };

    TokenStream::from(expanded)
}

/// This macro attribute wraps actix http route handlers, due to
/// the limitation where the attribute definition order is undefined
/// this macro also wraps the actix_web::{get, post, put, patch, delete, options, trace}
/// macros.
///
/// The usage in a route handler is the following.
///
/// ```ignore
/// use actix_error_proc::{ActixError, Error, HttpResult}; // Error is a thiserror re export.
/// use crate::models::user::User;
/// use actix_web::{main, App, HttpServer}
/// use serde_json::{from_slice, Error as JsonError};
/// use std::io::Error as IoError;
///
/// // assuming we have a wrapped enum
/// #[derive(ActixError, Error, Debug)]
/// enum SomeError {
///     #[error("The body could not be parsed.")]
///     #[status_code(BadRequest)]
///     InvalidBody(#[from] JsonError)
/// }
///
/// #[proof_route(post("/users"))]
/// async fn some_route(body: Bytes) -> HttpResult<SomeError> {
///     let user: User = from_slice(body)?; // notice the use of `?`.
///
///     // Do something with the user.
///
///     Ok(HttpResponse::NoContent()) // return Ok if everything went fine.
/// }
///
/// async fn main() -> IoError {
///     HttpServer::new(|| {
///         App::new()
///             .service(some_route) // we can register the route normally.
///     })
///         .bind(("0.0.0.0", 8080))?
///         .run()
///         .await
/// }
/// ```
#[proc_macro_attribute]
pub fn proof_route(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as ExprCall);
    let mut item = parse_macro_input!(item as ItemFn);

    let original_name = item.sig.ident.clone();
    let renamed_ident = Ident::new(
        &format!("__proof_route_{original_name}"),
        original_name.span(),
    );
    item.sig.ident = renamed_ident.clone();

    let allowed_methods = ["get", "put", "post", "delete", "patch", "options", "trace"];

    let method = if let Expr::Path(path) = *args.func {
        let method = path.to_token_stream().to_string();

        if allowed_methods.contains(&method.as_str()) {
            Ident::new(&method, path.span())
        } else {
            panic!("The method is not a valid HTTP method.");
        }
    } else {
        panic!("Expected a path.");
    };

    let path = if let Some(arg) = args.args.first() {
        if let Expr::Lit(ExprLit {
            lit: Lit::Str(path),
            ..
        }) = arg
        {
            path
        } else {
            panic!("Expected a string literal argument.");
        }
    } else {
        panic!("Expected at least one argument.");
    };

    if args.args.len() > 1 {
        panic!("Expected only one argument.");
    }

    let inputs = item.sig.inputs.to_token_stream();
    let input_names = item.sig.inputs.iter().map(|arg| {
        if let FnArg::Typed(pat_type) = arg {
            pat_type.pat.to_token_stream()
        } else {
            quote! { self }
        }
    });

    TokenStream::from(quote! {
        #[actix_web::#method(#path)]
        async fn #original_name(#inputs) -> impl actix_web::Responder {
            #item

            match #renamed_ident(#(#input_names),*).await {
                ::core::result::Result::Ok(r) => r,
                ::core::result::Result::Err(r) => r.into()
            }
        }
    })
}
