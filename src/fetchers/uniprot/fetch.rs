//! UniProt fetch functionality
//!
//! This module provides the fetch interface for retrieving a single UniProt entry by accession.

use anyhow::Result;
use enzymeml::prelude::Protein;

use crate::fetchers::{
    client::build_client,
    uniprot::{
        types::{UniProtEntry, UniProtField},
        url::build_fetch_url,
    },
};

const DEFAULT_FIELDS: &[UniProtField; 4] = &[
    UniProtField::Accession,
    UniProtField::Ec,
    UniProtField::OrganismName,
    UniProtField::Sequence,
];

/// Fetch a single protein entry from UniProt by accession number
///
/// This function retrieves a single UniProt entry using the REST API endpoint
/// `/uniprotkb/{accession}`. The response is a single `UniProtEntry` (not wrapped
/// in a `UniProtResponse`).
///
/// # Arguments
///
/// * `accession` - The UniProt accession number (e.g., "P05067")
/// * `fields` - Optional list of fields to retrieve. If None, defaults to
///   [Accession, Ec, OrganismName, Sequence]
///
/// # Returns
///
/// A TOON-encoded string containing a single `Protein` object.
///
/// # Errors
///
/// Returns an error if the REST API request fails, the network request fails,
/// or the response cannot be parsed.
pub async fn fetch_uniprot(accession: String) -> Result<Protein> {
    let client = build_client()?;
    let url = build_fetch_url(&accession, Some(DEFAULT_FIELDS));

    let response = client.get(&url).send().await?;

    if !response.status().is_success() {
        anyhow::bail!(
            "UniProt API request failed with status: {}",
            response.status()
        );
    }

    // Parse response as a single UniProtEntry (not UniProtResponse)
    let uniprot_entry: UniProtEntry = response.json().await?;

    Ok(uniprot_entry.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_uniprot() {
        let protein = fetch_uniprot("Z9JZ75".to_string()).await.unwrap();
        assert_eq!(protein.id, "Z9JZ75");
        assert!(!protein.name.is_empty());
        assert!(protein.ecnumber.is_some());
        assert!(protein.organism.is_some());
        assert!(protein.organism_tax_id.is_some());
        assert!(protein.sequence.is_some());
    }

    #[tokio::test]
    async fn test_fetch_protein_invalid_accession() {
        let result = fetch_uniprot("INVALID".to_string()).await;
        assert!(result.is_err());
    }
}
