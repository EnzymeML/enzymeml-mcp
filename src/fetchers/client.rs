//! HTTP client configuration for UniProt API
//!
//! This module handles building the HTTP client with appropriate headers and timeouts.

use anyhow::Result;
use reqwest::{
    Client,
    header::{ACCEPT, HeaderMap, HeaderValue},
};
use std::time::Duration;

/// Build HTTP client with appropriate headers
pub(crate) fn build_client() -> Result<Client, reqwest::Error> {
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
    Client::builder()
        .default_headers(headers)
        .timeout(Duration::from_secs(30))
        .build()
}
