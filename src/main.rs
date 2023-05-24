use reqwest::Client;
use salvo::prelude::*;
use util::REQWEST_CLIENT;
use version::VersionData;

pub mod routes;
pub mod util;
pub mod version;

#[handler]
async fn versions(res: &mut Response) {
    res.render(Json(VersionData::default()));
}

#[tokio::main]
async fn main() {
    REQWEST_CLIENT
        .set(Client::new())
        .expect("This should never error");

    let acceptor = TcpListener::new("127.0.0.1:5800").bind().await;
    Server::new(acceptor).serve(routes::built_routes()).await;
}
