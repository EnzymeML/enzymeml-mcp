//! ChEBI API data types
//!
//! This module contains the data structures used for interacting with
//! the ChEBI REST API, including search response types.

use serde::Deserialize;

/// ChEBI search API response structure
///
/// This represents the top-level response from a ChEBI search query.
/// The response contains an array of search results, where each result
/// includes metadata (index, type, ID, score) and the actual compound
/// data in the `_source` field.
#[derive(Debug, Deserialize)]
pub struct ChebiSearchResponse {
    /// Array of search results matching the query
    pub results: Vec<ChebiSearchResult>,
}

/// Individual search result from ChEBI search API
///
/// Each result contains Elasticsearch metadata fields (`_index`, `_type`,
/// `_id`, `_score`) along with the actual compound data in `_source`.
#[derive(Debug, Deserialize)]
pub struct ChebiSearchResult {
    /// The actual compound data
    #[serde(rename = "_source")]
    pub source: ChebiCompound,
}

/// ChEBI compound data structure
///
/// Contains all the chemical and biological information about a compound
/// from the ChEBI database, including identifiers, names, structures,
/// and physical properties.
#[derive(Debug, Clone, Deserialize)]
pub struct ChebiCompound {
    /// ChEBI accession number (e.g., "CHEBI:16236")
    #[serde(rename = "chebi_accession")]
    pub chebi_accession: String,
    /// Primary name of the compound
    pub name: String,
    /// ASCII representation of the compound name
    #[serde(rename = "ascii_name")]
    pub ascii_name: String,
    /// SMILES (Simplified Molecular Input Line Entry System) notation
    pub smiles: Option<String>,
    /// Formal charge of the compound
    pub charge: Option<i32>,
    /// Monoisotopic mass in atomic mass units
    pub monoisotopicmass: Option<f64>,
    /// InChI (International Chemical Identifier) string
    pub inchi: Option<String>,
    /// Average mass in atomic mass units
    pub mass: Option<f64>,
    /// Molecular formula
    pub formula: Option<String>,
    /// InChI key (hashed version of InChI)
    pub inchikey: Option<String>,
}
