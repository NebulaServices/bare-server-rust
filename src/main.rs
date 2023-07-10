use error::{Error, ErrorWithContext};
use miette::{Context, Diagnostic, ErrReport as Report, IntoDiagnostic};
use reqwest::Client;
use salvo::{__private::tracing, prelude::*};
//use util::REQWEST_CLIENT;
use version::VersionData;
mod v3;

pub mod error;
pub mod routes;
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
    tracing_subscriber::fmt::init();
    let app = Router::new()
        .hoop(Logger::new())
        .get(versions)
        .push(Router::with_path("error").get(error_test));
    let server = TcpListener::new("127.0.0.1:5800").bind().await;
    Server::new(server).serve(app).await;
}
