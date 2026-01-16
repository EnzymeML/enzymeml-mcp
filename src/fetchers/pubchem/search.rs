//! PubChem search functionality
//!
//! This module provides the search interface for querying the PubChem database
//! using the autocomplete API to find compound names matching search terms.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use urlencoding::encode;

use crate::fetchers::{client::build_client, pubchem::types::PubChemACResponse};

/// PubChem autocomplete API endpoint for compound searches
const PUBCHEM_AC_ENDPOINT: &str = "https://pubchem.ncbi.nlm.nih.gov/rest/autocomplete/compound/";

/// Search parameters for querying the PubChem autocomplete API
///
/// This structure defines the parameters needed to perform a compound name
/// search against the PubChem database. It supports configurable result limits
/// and automatic URL encoding of search terms.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct PubChemSearch {
    /// The search term to query for compound names
    #[schemars(description = "The term to search for")]
    pub term: String,
    /// Maximum number of results to return from the search
    #[schemars(description = "The number of results to return")]
    #[serde(default = "default_limit")]
    pub limit: usize,
}

/// Default limit for search results
///
/// Returns the default maximum number of search results when no limit is specified.
fn default_limit() -> usize {
    20
}

impl PubChemSearch {
    /// Performs an autocomplete search against the PubChem database
    ///
    /// Sends a request to the PubChem autocomplete API with the configured
    /// search term and limit, returning a list of matching compound names.
    ///
    /// # Returns
    /// - `Ok(Vec<String>)`: List of compound names matching the search term
    /// - `Err(anyhow::Error)`: Network error, API error, or parsing failure
    ///
    /// # Errors
    /// This function will return an error if:
    /// - The HTTP client cannot be built
    /// - The network request fails
    /// - The PubChem API returns a non-success status code
    /// - The response cannot be parsed as valid JSON
    pub async fn search(&self) -> anyhow::Result<Vec<String>> {
        let client = build_client()?;
        let url = self.build_url();
        let response = client.get(&url).send().await?;

        if !response.status().is_success() {
            anyhow::bail!(
                "PubChem API request failed with status: {}",
                response.status()
            );
        }

        let pubchem_response: PubChemACResponse = response.json().await?;
        Ok(pubchem_response.dictionary_terms.compound)
    }

    /// Constructs the API URL for the autocomplete request
    ///
    /// Builds the complete URL for the PubChem autocomplete API request,
    /// including URL encoding of the search term and the result limit parameter.
    ///
    /// # Returns
    /// A formatted URL string ready for HTTP requests
    fn build_url(&self) -> String {
        let encoded_term = encode(&self.term);
        let limit = self.limit.to_string();
        format!("{PUBCHEM_AC_ENDPOINT}/{encoded_term}/json?limit={limit}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_search() {
        let query = PubChemSearch {
            term: "ethanol".to_string(),
            limit: 10,
        };
        let results = query.search().await.unwrap();
        assert!(!results.is_empty());
    }
}
