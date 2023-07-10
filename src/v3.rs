use std::any::Any;

use crate::{error, util};
use reqwest::header::HeaderMap;
use salvo::prelude::*;

pub struct ProcessedData {}

#[handler]
pub async fn process_headers(req: &mut Request, depot: &mut Depot) -> error::Result<()> {
    let headers: &mut HeaderMap = req.headers_mut();
    let bare_headers = util::join_bare_headers(&headers);
    let mut forward: Option<HeaderMap> = None;
    if let Some(forward_headers) = headers.get("X-BARE-FORWARD-HEADERS") {}

    todo!()
}

#[handler]
pub async fn fetch() -> &'static str {
    todo!()
}
