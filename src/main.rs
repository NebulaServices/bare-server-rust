use error::{Error, ErrorWithContext};
use reqwest::Client;
use salvo::{__private::tracing, prelude::*};
use util::REQWEST_CLIENT;
use v3::add_cors_headers_route;
//use util::REQWEST_CLIENT;
use version::VersionData;
#[macro_use]
extern crate cfg_if;
mod v3;

pub mod error;
pub mod util;
pub mod version;

#[handler]
async fn versions() -> Json<VersionData> {
    Json(VersionData::default())
}

#[handler]
async fn error_test() -> Result<(), ErrorWithContext> {
    let report = Err(ErrorWithContext::new(
        Error::Unknown,
        "While testing errors.",
    ));
    report?
}

#[tokio::main]
async fn main() {
    REQWEST_CLIENT
        .set(Client::new())
        .expect("This should never error");
    tracing_subscriber::fmt::init();
    tracing::info!("We gucchi");

    // Compiler will complain.
    #[allow(unused_mut)]
    let mut app = Router::new()
        //.hoop(Logger::new())
        .hoop(add_cors_headers_route)
        .get(versions)
        .push(
            Router::with_path("v3")
                .hoop(crate::v3::process_headers)
                .handle(crate::v3::fetch),
        )
        .push(Router::with_path("error").get(error_test));
    cfg_if! {
        if #[cfg(feature = "v2")] {
            app = app.push(
                Router::with_path("v2")
                    .hoop(crate::v3::process_headers)
                    .handle(crate::v3::fetch),
            );
        }
    }
    let server = TcpListener::new("127.0.0.1:5800").bind().await;
    Server::new(server).serve(app).await;
}
