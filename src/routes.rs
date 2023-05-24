use reqwest::header::HeaderMap;

use salvo::http::header::{HeaderName, HeaderValue};

use salvo::http::request::secure_max_size;
use salvo::prelude::*;


use crate::util::{join_bare_headers, split_headers, ProcessedHeaders, REQWEST_CLIENT};
use crate::version::VersionData;
use std::any::TypeId;
use std::collections::HashMap;
use std::ops::DerefMut;
use std::str::FromStr;

#[handler]
async fn versions(res: &mut Response) {
    add_cors_headers(res);
    res.render(Json(VersionData::default()));
}

#[handler]
/// A function to preprocess headers from a request and inject them to the depot
async fn preprocess_headers(req: &mut Request, depot: &mut Depot) {
    // Get a mutable reference to the headers from the request
    let headers: &mut HeaderMap = req.headers_mut();
    // Create a new processed headers object with default values
    let mut processed = ProcessedHeaders::default();

    // Process forwarded headers using functional methods
    // Get the value of x-bare-forward-headers or use an empty string as default
    let header_value = headers.get("x-bare-forward-headers").map_or("", |h| {
        h.to_str().expect("Should map to string successfully")
    });
    // Split the value by comma and space and collect it into a vector
    let forwarded_heads: Vec<String> = header_value.split(", ").map(|s| s.to_owned()).collect();
    // Filter out the invalid headers and append the valid ones to the processed
    // headers
    forwarded_heads
        .iter()
        .filter(|head| {
            match head.as_str() {
                // If headers are invalid, we don't need them.
                "connection" | "transfer-encoding" | "host" | "origin" | "referer" | "" => false,
                _ => true,
            }
        })
        .for_each(|head| {
            println!("Current head: {head}");
            let he = &HeaderName::from_str(head).expect("Should not fail here");
            processed.append(he, headers.get(he).expect("Header should exist").clone());
        });

    // Get the value of x-bare-headers or use the joined bare headers as default
    let bare_headers = headers
        .get("x-bare-headers")
        .map_or_else(|| join_bare_headers(headers).unwrap(), |h| h.to_owned());

    // Process bare headers if they exist
    if !bare_headers.is_empty() {
        // Deserialize the bare headers into a hashmap of strings
        let data: HashMap<String, String> =
            serde_json::from_str(bare_headers.to_str().expect("Should be valid string"))
                .expect("Should not fail to Str:Str deserialize");
        // Append the hashmap entries to the processed headers
        data.iter().for_each(|(head, value)| {
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
        // Host key is not needed, I think?
        processed.remove("host");
    }
    // Inject processed headers to the depot.
    depot.inject(processed);
}

#[handler]
/// Handler for [`TOMPHttp V2`](https://github.com/tomphttp/specifications/blob/master/BareServerV2.md#send-and-receive-data-from-a-remote) requests.
async fn v2_get(req: &mut Request, res: &mut Response, depot: &mut Depot) {
    // Get a mutable reference to the processed headers from the depot
    let headers: &mut ProcessedHeaders = depot
        .get_mut(&format!("{:?}", TypeId::of::<ProcessedHeaders>()))
        .unwrap();

    // Get the path from the request header or use "/" as default
    let path = req
        .header::<String>("x-bare-path")
        .unwrap_or("/".to_owned());

    // Construct the full URL from the request header or use default values
    let url = format!(
        "{}//{}{}",
        // Assume HTTPS if not specified
        req.header::<String>("x-bare-protocol")
            .unwrap_or("https:".to_owned()),
        req.header::<String>("x-bare-host")
            .unwrap_or("example.com".to_owned()),
        path
    );

    // Make a new request using the same method and URL as the original one
    let response = REQWEST_CLIENT
        .get()
        .unwrap()
        .request(req.method().clone(), url)
        // Use the processed headers as the new request headers
        .headers(headers.deref_mut().to_owned())
        // Read the payload from the original request with a maximum size limit
        .body(
            req.payload_with_max_size(secure_max_size())
                .await
                .expect("Probably won't error?")
                .to_vec(),
        )
        // Send the new request and panic if it fails
        .send()
        .await
        .unwrap_or_else(|x| {
            panic!("{x}");
        });

    // Set the status code of the response to match the new request's status code
    res.status_code(response.status());
    // Set x-bare-headers to show the new request's headers
    res.add_header("x-bare-headers", format!("{:?}", response.headers()), true)
        .expect("This shouldn't fail, probably?");
    // Split the headers if needed
    res.set_headers(split_headers(&res.headers));
    // Set some of the required headers from the new request's headers
    if let Some(header) = response.headers().get("content-type") {
        res.add_header("content-type", header, true).unwrap();
    }
    res.add_header("x-bare-status", response.status().as_str(), true)
        .expect("This shouldn't fail.");
    res.add_header(
        "x-bare-status-text",
        response.status().canonical_reason().expect("Should exist"),
        true,
    )
    .expect("This shouldn't fail");
    // Add cors headers to the response
    add_cors_headers(res);

    // Write the body of the response using the bytes from the new request's
    // response
    res.write_body(response.bytes().await.unwrap())
        .expect("This should not fail?");
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

/// Build our routes.
pub fn built_routes() -> Router {
    Router::new().get(versions).push(
        Router::with_path("v2")
            .hoop(preprocess_headers)
            .handle(v2_get),
    )
}
