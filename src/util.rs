use once_cell::sync::OnceCell;
use reqwest::Client;
use salvo::{
    http::HeaderValue,
    hyper::{http::HeaderName, HeaderMap},
};
use std::{
    ops::{Deref, DerefMut},
    str::{self, FromStr},
};
const MAX_HEADER_VALUE: usize = 3072;
pub static REQWEST_CLIENT: OnceCell<Client> = OnceCell::new();
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
pub fn join_bare_headers(headers: &HeaderMap) -> Result<HeaderValue, String> {
    let mut err: Option<String> = None;
    // Create a new empty string for the joined header value
    let mut joined_value = String::new();
    // Iterate over each header in the input header map
    headers.iter().for_each(|(name, value)| {
        // Check if the header name has the prefix "x-bare-headers-" (case-insensitive)
        if name.as_str().to_lowercase().starts_with("x-bare-headers-") {
            if !value
                .to_str()
                .expect("[Join Headers] Should be convertable to string")
                .starts_with(';')
            {
                err = Some("Header started with invalid character.".into());
            }
            // Append the header value to the joined value string
            joined_value.push_str(
                value
                    .to_str()
                    .expect("[Join Headers] Failed to convert header value to string?"),
            );
        }
    });
    if let Some(e) = err {
        return Err(e);
    }
    // Create a new header value from the joined value string
    let joined_value = HeaderValue::from_str(&joined_value)
        .expect("[Join Headers] Failed to create header value?");
    // Return joined values
    Ok(joined_value)
}
