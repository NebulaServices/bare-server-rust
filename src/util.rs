use std::{
    ops::{Deref, DerefMut},
    str::{self, FromStr},
};

use once_cell::sync::OnceCell;
use reqwest::Client;
use salvo::{
    http::HeaderValue,
    hyper::{http::HeaderName, HeaderMap},
};

use crate::error::{Error, ErrorWithContext};
const MAX_HEADER_VALUE: usize = 3072;
pub static REQWEST_CLIENT: OnceCell<Client> = OnceCell::new();

#[derive(Default, Clone, Debug)]
pub struct RequestData{
    pub processed_headers: HeaderMap,
    pub pass_headers: Option<Vec<String>>,
    pub status: Option<Vec<String>> 
}

impl RequestData {
    pub fn explode_ref_mut(&mut self) -> (&mut HeaderMap, Option<&mut Vec<String>>, Option<&mut Vec<String>>) {
        (&mut self.processed_headers, self.pass_headers.as_mut(), self.status.as_mut())
    }
}

#[derive(Default, Clone, Debug)]
pub struct ProcessedHeaders(HeaderMap);

impl Deref for ProcessedHeaders {
    type Target = HeaderMap;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ProcessedHeaders {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// This function splits any header value in the input HeaderMap that is longer
/// than MAX_HEADER_VALUE (3072 bytes) into smaller headers with a suffix
/// indicating their order and returns a new HeaderMap.
///
/// For example, a header "X-BARE-HEADERS" with a 5000-byte value will be split
/// into two headers: "X-BARE-HEADERS-0" and "X-BARE-HEADERS-1".
///
/// The original case of the header names is preserved and other headers are not
/// modified.
pub fn split_headers(headers: &HeaderMap) -> HeaderMap {
    // Create a new empty header map for the output
    let mut output = HeaderMap::new();
    // Iterate over each header in the input header map
    headers.iter().for_each(|(name, value)| {
        // Check if the header name is "x-bare-headers" (case-insensitive) and if the
        // header value length exceeds the MAX_HEADER_VALUE limit
        if name.as_str().to_lowercase() == "x-bare-headers" && value.len() > MAX_HEADER_VALUE {
            // Split the header value into chunks of MAX_HEADER_VALUE bytes
            value
                .as_bytes()
                .chunks(MAX_HEADER_VALUE)
                // Convert each chunk into a string slice
                .map(|buf| unsafe { str::from_utf8_unchecked(buf) })
                // Enumerate each chunk with an index
                .enumerate()
                // For each chunk, create a new header name with the suffix "-{i}" where i is the
                // index, and append it to the output header map with the chunk as the value
                .for_each(|(i, value)| {
                    output.append(
                        HeaderName::from_str(&format!("X-BARE-HEADERS-{i}"))
                            .expect("[Split Headers] Failed to create header name?"),
                        HeaderValue::from_str(&format!(";{value}"))
                            .expect("[Split Headers] Failed to split header content?"),
                    );
                });
        } else {
            // If the header name is not "x-bare-headers" or the header value length is
            // within the limit, append it to the output header map as it is
            output.append(name, value.clone());
        }
    });
    // Return the output header map
    output
}

/// This function joins any headers in the input HeaderMap that have the prefix
/// "X-BARE-HEADERS-" into a single header with the name "X-BARE-HEADERS" and
/// returns a new HeaderMap.
///
/// For example, two headers "X-BARE-HEADERS-0" and "X-BARE-HEADERS-1" with
/// values "foo" and "bar" respectively will be joined into one header
/// "X-BARE-HEADERS" with the value "foobar".
///
/// The original case of the header names is preserved and other headers are not
/// modified.
pub fn join_bare_headers(headers: &HeaderMap) -> Result<HeaderValue, ErrorWithContext> {
    // Create an early out if `x-bare-headers` exists on its own
    if let Some(header) = headers.get("x-bare-headers") {
        return Ok(header.to_owned());
    }
    // Create a new empty string for the joined header value
    let mut joined_value = String::new();
    // Why couldn't they have used duplicate headers.
    // It'd be less ugly. Oh well.
    let mut x = 0;
    while let Some(header) = headers.get(format!("x-bare-headers-{x}")) {
        joined_value.push_str(
            header
                .to_str()
                .expect("[Join Headers] Failed to convert header value to string?"),
        );
        x += 1;
    }

    // We can assume that this header was never specified..
    if joined_value.is_empty() && x != 0 {
        Err(ErrorWithContext::new(
            Error::MissingBareHeader("X-BARE-HEADERS".into()),
            "While joining headers",
        ))?
    }

    // Create a new header value from the joined value string
    let joined_value = HeaderValue::from_str(&joined_value).unwrap();
    // Return joined values
    Ok(joined_value)
}
