//! URL building for UniProt API requests
//!
//! This module handles constructing the REST API URL with query parameters.

use crate::fetchers::uniprot::search::ProteinSearch;
use crate::fetchers::uniprot::types::UniProtField;
use urlencoding::encode;

const UNIPROT_REST_ENDPOINT: &str = "https://rest.uniprot.org/uniprotkb/search";
const UNIPROT_FETCH_ENDPOINT_BASE: &str = "https://rest.uniprot.org/uniprotkb";

/// Build the REST API URL with query parameters
pub(crate) fn build_url(search: &ProteinSearch) -> String {
    let encoded_query = encode(&search.query);

    // Convert fields enum to comma-separated string
    let fields = match &search.fields {
        Some(field_vec) => field_vec
            .iter()
            .map(|f| f.to_api_string())
            .collect::<Vec<_>>()
            .join(","),
        None => {
            // Default fields: accession,ec,organism_name,protein_name,sequence
            [
                UniProtField::Accession,
                UniProtField::Ec,
                UniProtField::OrganismName,
                UniProtField::ProteinName,
                UniProtField::Sequence,
            ]
            .iter()
            .map(|f| f.to_api_string())
            .collect::<Vec<_>>()
            .join(",")
        }
    };

    let size = search.limit.to_string();
    let sort = search.sort.unwrap_or_default().to_api_string();

    format!(
        "{}?query={}&fields={}&sort={}&size={}&compressed=false",
        UNIPROT_REST_ENDPOINT,
        encoded_query,
        encode(&fields),
        encode(&sort),
        size
    )
}

/// Build the REST API URL for fetching a single entry by accession
pub(crate) fn build_fetch_url(accession: &str, fields: Option<&[UniProtField]>) -> String {
    // Convert fields enum to comma-separated string
    let fields_str = match fields {
        Some(field_vec) => field_vec
            .iter()
            .map(|f| f.to_api_string())
            .collect::<Vec<_>>()
            .join(","),
        None => {
            // Default fields: accession,ec,organism_name,sequence
            [
                UniProtField::Accession,
                UniProtField::Ec,
                UniProtField::OrganismName,
                UniProtField::Sequence,
            ]
            .iter()
            .map(|f| f.to_api_string())
            .collect::<Vec<_>>()
            .join(",")
        }
    };

    format!(
        "{}/{}?fields={}",
        UNIPROT_FETCH_ENDPOINT_BASE,
        encode(accession),
        encode(&fields_str)
    )
}
