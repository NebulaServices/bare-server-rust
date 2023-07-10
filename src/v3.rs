use std::{any::TypeId, collections::HashMap, str::FromStr};

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use salvo::{http::request::secure_max_size, prelude::*};

use crate::{
    error::{self, Error, ErrorWithContext},
    util::{self, split_headers, RequestData, REQWEST_CLIENT},
};

#[handler]
pub async fn process_headers(req: &mut Request, depot: &mut Depot) -> error::Result<()> {
    let headers: &mut HeaderMap = req.headers_mut();
    let bare_headers = util::join_bare_headers(headers)?;
    let mut processed = HeaderMap::new();
    headers
        .get("X-BARE-FORWARD-HEADERS")
        .get_or_insert(&HeaderValue::from_str("").unwrap())
        .to_str()
        .expect("Should not fail")
        .split(", ")
        .collect::<Vec<&str>>()
        .iter()
        .filter(|head| match **head {
            // If headers are invalid, we don't need them.
            "connection" | "transfer-encoding" | "host" | "origin" | "referer" | "" => false,
            _ => true,
        })
        .for_each(|head| {
            let he = &HeaderName::from_str(head).expect("Should not fail here");
            processed.append(he, headers.get(he).expect("Header should exist").clone());
        });

    let bare_header_map: HashMap<String, String> =
        serde_json::from_str(bare_headers.to_str().expect("Should be valid string"))
            .unwrap_or(HashMap::new());

    // Append the hashmap entries to the processed headers
    bare_header_map.iter().for_each(|(head, value)| {
        processed.append(
            HeaderName::from_str(head).unwrap(),
            HeaderValue::from_str(value).unwrap(),
        );
    });

    // Pass content length header too.
    if let Some(content_length) = headers.get("content-length") {
        processed.append(
            HeaderName::from_str("content-length").unwrap(),
            content_length.to_owned(),
        );
    }

    // We don't need the host key, can cause issues if specified improperly.
    processed.remove("host");

    depot.inject(RequestData {
        processed_headers: processed,
        pass_headers: None,
        status: None,
    });
    Ok(())
}

#[handler]
pub async fn fetch(req: &mut Request, depot: &mut Depot, resp: &mut Response) -> error::Result<()> {
    let (headers, pass_headers, statuses) = depot
        .get_mut::<RequestData>(&format!("{:?}", TypeId::of::<RequestData>()))
        .unwrap()
        .explode_ref_mut();

    cfg_if! {
        if #[cfg(feature = "v2")] {
            let url = req.header::<String>("x-bare-url").unwrap_or(format!(
                "{}//{}{}",
                // Assume HTTPS if not specified
                req.header::<String>("x-bare-protocol")
                    .unwrap_or("https:".to_owned()),
                req.header::<String>("x-bare-host")
                    .unwrap_or("example.com".to_owned()),
                req.header::<String>("x-bare-path")
                    .unwrap_or("/".to_owned())
            ));
        } else {
            let url = req.header::<String>("x-bare-url")
                .ok_or(ErrorWithContext::new(Error::MissingBareHeader("x-bare-url".into()), "While processing v3 request."))?;
        }
    }

    let response = REQWEST_CLIENT
        .get()
        .unwrap()
        .request(req.method().clone(), url)
        // Use the processed headers as the new request headers
        .headers(headers.to_owned())
        // Read the payload from the original request with a maximum size limit
        .body(
            req.payload_with_max_size(secure_max_size())
                .await
                .map_err(|_| {
                    ErrorWithContext::new(
                        Error::Generic("invalid_body".into()),
                        "Couldn't parse request body.",
                    )
                })?
                .to_vec(),
        )
        .send()
        .await
        .map_err(|e| {
            ErrorWithContext::new(Error::Generic("unhandled_error".into()), e.to_string())
        })?;

    resp.add_header("x-bare-headers", format!("{:?}", response.headers()), true)
        .expect("This shouldn't fail, probably?");
    // Split the headers if needed
    resp.set_headers(split_headers(&resp.headers));

    if statuses.is_some() && statuses.unwrap().contains(&response.status().to_string()) {
        resp.status_code(response.status());
    }

    if pass_headers.is_some() {
        pass_headers.unwrap().iter().for_each(|header| {
            if let Some(value) = response.headers().get(header) {
                resp.headers
                    .append(HeaderName::from_str(header).unwrap(), value.clone());
            }
        });
    }

    // We should ALWAYS copy the content type.
    if let Some(header) = response.headers().get("content-type") {
        resp.add_header("content-type", header, true).unwrap();
    }
    resp.add_header("x-bare-status", response.status().as_str(), true)
        .expect("Should never fail to add `x-bare-status`");

    resp.add_header(
        "x-bare-status-text",
        response
            .status()
            .canonical_reason()
            .expect("canonical_reason should always exist."),
        true,
    )
    .expect("Should never fail to add `x-bare-status-text`");

    add_cors_headers(resp);

    resp.write_body(response.bytes().await.unwrap())
        .expect("This should not fail?");
    Ok(())
}

/// Blanket fix for CORS headers while in dev.
///
/// THIS IS BAD AND A SEC VULN, WILL BE FIXED LATER.
fn add_cors_headers(res: &mut Response) {
    res.add_header("access-control-allow-origin", "*", true)
        .unwrap();
    res.add_header("access-control-allow-headers", "*", true)
        .unwrap();
    res.add_header("access-control-allow-methods", "*", true)
        .unwrap();
    res.add_header("access-control-expose-headers", "*", true)
        .unwrap();
}

/// Blanket fix for CORS headers while in dev.
///
/// THIS IS BAD AND A SEC VULN, WILL BE FIXED LATER.
#[handler]
pub fn add_cors_headers_route(res: &mut Response) {
    res.add_header("access-control-allow-origin", "*", true)
        .unwrap();
    res.add_header("access-control-allow-headers", "*", true)
        .unwrap();
    res.add_header("access-control-allow-methods", "*", true)
        .unwrap();
    res.add_header("access-control-expose-headers", "*", true)
        .unwrap();
}
