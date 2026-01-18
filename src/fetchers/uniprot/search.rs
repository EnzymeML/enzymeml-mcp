//! UniProt search functionality
//!
//! This module provides the search interface for querying the UniProt database.

use anyhow::Result;
use enzymeml::prelude::Protein;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use toon_format::encode_default;

use crate::fetchers::{
    client::build_client,
    uniprot::{
        types::{SortOption, UniProtField, UniProtResponse},
        url::build_url,
    },
};

/// Search parameters for UniProt protein queries
///
/// This structure defines the parameters needed to search the UniProt database
/// for proteins. Supports complex Boolean queries and field-based filtering.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[schemars(description = "Search parameters for querying the UniProt database. Supports Boolean operators (AND, OR, NOT) and field-based filtering.")]
pub struct ProteinSearch {
    /// Query string to search for (supports AND/NOT operators)
    #[schemars(description = "The search query string. Supports Boolean operators (AND, OR, NOT) and field-based filtering (e.g., 'ec:1.1.1.1 AND organism_name:human'). See https://rest.uniprot.org/configure/uniprotkb/search-fields for available fields.")]
    pub query: String,
    /// Maximum number of results to return (default: 10)
    #[schemars(description = "Maximum number of search results to return. Defaults to 10 if not specified.")]
    pub limit: usize,
    /// Fields to retrieve (default: [Accession, Ec, OrganismName, ProteinName, Sequence])
    #[schemars(description = "Specific fields to retrieve from UniProt. If not specified, defaults to: Accession, EC number, OrganismName, ProteinName, and Sequence.")]
    pub fields: Option<Vec<UniProtField>>,
    /// Sort order (default: Accession Desc)
    #[schemars(description = "Sort order for the results. If not specified, defaults to sorting by Accession in descending order.")]
    pub sort: Option<SortOption>,
}

/// Search for proteins in the UniProt database using REST API
///
/// This function performs a REST API query against the UniProt database to find
/// proteins matching the given query string. The query supports AND/NOT operators
/// and is URL-encoded automatically.
///
/// # Arguments
///
/// * `search` - The search parameters specifying the query string and options
///
/// # Returns
///
/// A vector of `Protein` objects constructed from the UniProt results.
///
/// # Errors
///
/// Returns an error if the REST API request fails, the network request fails,
/// or the response cannot be parsed.
pub async fn search_uniprot(search: ProteinSearch) -> Result<String> {
    let client = build_client()?;
    let url = build_url(&search);

    let response = client.get(&url).send().await?;

    if !response.status().is_success() {
        anyhow::bail!(
            "UniProt API request failed with status: {}",
            response.status()
        );
    }

    // Use response.json() - reqwest should handle decompression automatically
    let uniprot_response: UniProtResponse = response.json().await?;

    let proteins: Vec<Protein> = uniprot_response
        .results
        .into_iter()
        .map(|entry| entry.into())
        .collect::<Vec<Protein>>();

    Ok(encode_default(&proteins)?.to_string())
}
