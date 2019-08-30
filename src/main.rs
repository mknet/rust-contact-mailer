extern crate futures;
extern crate gotham;
extern crate hyper;
extern crate serde_json;
extern crate serde;
extern crate dotenv;

use futures::{future, Future, Stream};
use gotham::handler::{HandlerFuture, IntoHandlerError};
use gotham::helpers::http::response::create_empty_response;
use gotham::router::builder::{build_simple_router, DefineSingleRoute, DrawRoutes};
use gotham::router::Router;
use gotham::state::{FromState, State};
use hyper::{Body, HeaderMap, Method, StatusCode, Uri, Version};

mod mail;

fn print_request_elements(state: &State) {
    let method = Method::borrow_from(state);
    let uri = Uri::borrow_from(state);
    let http_version = Version::borrow_from(state);
    let headers = HeaderMap::borrow_from(state);
    println!("Method: {:?}", method);
    println!("URI: {:?}", uri);
    println!("HTTP Version: {:?}", http_version);
    println!("Headers: {:?}", headers);
}

fn post_handler(mut state: State) -> Box<HandlerFuture> {
    print_request_elements(&state);
    let f = Body::take_from(&mut state)
        .concat2()
        .then(|full_body| match full_body {
            Ok(valid_body) => {
                let body_content = String::from_utf8(valid_body.to_vec()).unwrap();
                println!("Body: {}", body_content);

                let mail_data: mail::ContactMail = serde_json::from_str(body_content.as_str()).unwrap();

                mail::send_contact_mail(mail_data);
                let res = create_empty_response(&state, StatusCode::OK);
                future::ok((state, res))
            }
            Err(e) => future::err((state, e.into_handler_error())),
        });

    Box::new(f)
}

fn router() -> Router {
    build_simple_router(|route| {
        route.post("/").to(post_handler);
    })
}

fn main() {
    for (key, value) in dotenv::vars() {
        println!("key: {}, value: {}", key, value)
    }

    let addr = "0.0.0.0:7878";
    gotham::start(addr, router())
}
