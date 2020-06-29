extern crate dotenv;
extern crate env_logger;
extern crate futures;
extern crate gotham;
extern crate hyper;
extern crate log;
extern crate serde;
extern crate serde_json;

use futures::{future, Future, Stream};
use gotham::handler::{HandlerFuture, IntoHandlerError};
use gotham::helpers::http::response::create_empty_response;
use gotham::router::builder::{build_simple_router, DefineSingleRoute, DrawRoutes};
use gotham::router::Router;
use gotham::state::{FromState, State};
use hyper::header::HeaderValue;
use hyper::{Body, Chunk, HeaderMap, Method, Response, StatusCode, Uri, Version};
use log::info;

mod mail;

const SMTP_PASSWORD_KEY: &'static str = "MK_RUST_MAILER_SMTP_PASSWORD";

trait Mailer{
    fn send_mail(&self, cfg: mail::Config, mail: mail::ContactMail);
}

struct WhateverMailer;
struct DummyMailer;

impl Mailer for WhateverMailer{
    fn send_mail(&self, cfg: mail::Config, mail: mail::ContactMail){
        mail::send_contact_mail(cfg,mail);
    }
}
impl Mailer for DummyMailer{
    fn send_mail(&self, cfg: mail::Config, mail: mail::ContactMail){
        unimplemented!();
    }
}

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
            Ok(valid_body) => future::ok(handle_valid_body(valid_body, state, DummyMailer)),
            Err(e) => future::err((state, e.into_handler_error())),
        });

    Box::new(f)
}

fn handle_valid_body<M: Mailer>(body: Chunk, state: State, mailer: M) -> (State, Response<Body>) {
    let smtp_password = dotenv::var(SMTP_PASSWORD_KEY).unwrap();

    let mail_config = mail::Config {
        password: smtp_password,
    };

    let body_content = String::from_utf8(body.to_vec()).unwrap();
    println!("Body: {}", body_content);

    let mail_data: mail::ContactMail = serde_json::from_str(body_content.as_str()).unwrap();

    mailer.send_mail(mail_config, mail_data);

    //send_fn(mail_config, mail_data);
    let mut res = create_empty_response(&state, StatusCode::OK);
    {
        let headers = res.headers_mut();
        headers.insert(
            "Access-Control-Allow-Origin",
            "https://www.marcelkoch.net".parse().unwrap(),
        );
        headers.insert(
            "Access-Control-Allow-Methods",
            "POST, OPTIONS, HEAD".parse().unwrap(),
        );
        headers.insert(
            "Access-Control-Allow-Headers",
            "Origin, Content-Type, X-Auth-Token".parse().unwrap(),
        );
    };

    (state, res)
}

pub fn options_handler(state: State) -> (State, Response<Body>) {
    let mut res = create_empty_response(&state, StatusCode::OK);

    let request_headers = HeaderMap::borrow_from(&state);
    let origin_header = request_headers.get("origin");
    // let header_value_to_check = HeaderValue::from_static(r"https:\/\/(.*\.)?marcelkoch\.net");
    match origin_header {
        Some(header_value_to_check)
            if header_value_to_check
                .to_str()
                .unwrap()
                .ends_with("marcelkoch.net") =>
        {
            let response_headers = res.headers_mut();
            response_headers.insert(
                "Access-Control-Allow-Origin",
                HeaderValue::from(origin_header.unwrap()),
            );
            response_headers.insert(
                "Access-Control-Allow-Methods",
                "POST, OPTIONS, HEAD".parse().unwrap(),
            );
            response_headers.insert(
                "Access-Control-Allow-Headers",
                "Origin, Content-Type, X-Auth-Token".parse().unwrap(),
            );
        }
        _ => {}
    }

    (state, res)
}

fn router() -> Router {
    build_simple_router(|route| {
        route.post("/").to(post_handler);
        route.options("/").to(options_handler);
    })
}

fn main() {
    env_logger::init();

    info!("Starting MK Rust Contact Mailer, listening on port: 7878");

    let addr = "0.0.0.0:7878";
    gotham::start(addr, router())
}

#[cfg(test)]
mod tests {

    use super::*;
    use mail::ContactMail;
    use mail::Config;

    #[test]
    fn test_handler() {
        
        fn dummy_send(config: Config, mail_data: ContactMail) {
            // do nothing
        }

        State::with_new(|state| {
            handle_valid_body(hyper::Chunk::from(""), state, DummyMailer);
        });

    }
}
