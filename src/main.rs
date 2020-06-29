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
            Ok(valid_body) => future::ok({

                let mail_res = handle_valid_body(valid_body, DummyMailer);

                /* we can now match on the mail_res and optionally handle the error
                let response_prototype = match mail_res {
                    Ok(_) => {
                        create an ok response
                    }
                    Err(_) => {
                        create a 400 or 500 error, depnding on the error type
                    }
                }
                */

                // we might as well re-use the response_prototype from above and enrich it here
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
            }),
            Err(e) => future::err((state, e.into_handler_error())),
        });

    Box::new(f)
}

// the handle body function is no longer concerned with handling http details, we therefore don't need to pass the state
fn handle_valid_body<M: Mailer>(body: Chunk, mailer: M) -> Result<(),()> {  // in a future iteration we should introduce Error types here

    // it might be a good idea to do this once at startup and save it either in a (lazy) static or in the gotham state
    let smtp_password = dotenv::var(SMTP_PASSWORD_KEY).unwrap();

    let mail_config = mail::Config {
        password: smtp_password,
    };

    let body_content = String::from_utf8(body.to_vec()).unwrap();
    println!("Body: {}", body_content);

    let mail_data: mail::ContactMail = serde_json::from_str(body_content.as_str()).unwrap();

    mailer.send_mail(mail_config, mail_data);

    // sending mail might fail (asynchonously), we might want to handle/map the result
    // for now it always works:
    Ok(())

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

        // we don't need the state here anymore
        handle_valid_body(hyper::Chunk::from(""), DummyMailer);

    }
}
