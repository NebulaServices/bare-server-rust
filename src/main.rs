use reqwest::Client;
use salvo::prelude::*;
use util::REQWEST_CLIENT;
use version::VersionData;
mod routes;
mod util;
mod version;
#[handler]
async fn versions(res: &mut Response) {
    res.render(Json(VersionData::default()));
}

// TODO: See https://github.com/tomphttp/specifications/blob/master/BareServerV2.md#send-and-receive-data-from-a-remote
#[handler]
async fn v2_get(_res: &mut Response) {
    todo!()
}

#[tokio::main]
async fn main() {
    REQWEST_CLIENT
        .set(Client::new())
        .expect("This should never error");
    let _router = Router::new()
        .get(versions)
        .push(Router::with_path("v2").get(v2_get));
    let acceptor = TcpListener::new("127.0.0.1:5800").bind().await;
    Server::new(acceptor).serve(routes::built_routes()).await;
}
