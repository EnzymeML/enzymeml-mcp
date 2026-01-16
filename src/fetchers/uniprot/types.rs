//! UniProt API data types and enums
//!
//! This module contains the data structures and enums used for interacting with
//! the UniProt REST API, including response types and field/sort configurations.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// UniProt REST API response structure
#[derive(Debug, Deserialize)]
pub struct UniProtResponse {
    pub results: Vec<UniProtEntry>,
}

/// Individual protein entry from UniProt REST API
#[derive(Debug, Deserialize)]
pub struct UniProtEntry {
    #[serde(rename = "primaryAccession")]
    pub primary_accession: String,
    #[serde(default)]
    pub organism: Option<Organism>,
    #[serde(rename = "proteinDescription", default)]
    pub protein_description: Option<ProteinDescription>,
    #[serde(default)]
    pub sequence: Option<Sequence>,
}

/// Organism information
#[derive(Debug, Deserialize)]
pub struct Organism {
    #[serde(rename = "scientificName")]
    pub scientific_name: String,
    #[serde(rename = "taxonId")]
    pub taxon_id: u64,
}

/// Protein description containing recommended name
#[derive(Debug, Deserialize)]
pub struct ProteinDescription {
    #[serde(rename = "recommendedName", default)]
    pub recommended_name: Option<RecommendedName>,
}

/// Recommended name structure
#[derive(Debug, Deserialize)]
pub struct RecommendedName {
    #[serde(rename = "fullName")]
    pub full_name: FullName,
    #[serde(rename = "ecNumbers", default)]
    pub ec_numbers: Option<Vec<EcNumberEntry>>,
}

/// Full name with value
#[derive(Debug, Deserialize)]
pub struct FullName {
    pub value: String,
}

/// Sequence information
#[derive(Debug, Deserialize)]
pub struct Sequence {
    pub value: String,
}

/// EC number entry
#[derive(Debug, Deserialize)]
pub struct EcNumberEntry {
    pub value: String,
}

/// Available fields that can be retrieved from UniProt entries
///
/// These are the fields that are actually extracted and used in the results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum UniProtField {
    /// Primary accession number
    Accession,
    /// EC (Enzyme Commission) number
    Ec,
    /// Organism scientific name
    OrganismName,
    /// Protein name (recommended name)
    ProteinName,
    /// Protein amino acid sequence
    Sequence,
}

impl UniProtField {
    /// Convert the field enum to its API string representation
    pub(crate) fn to_api_string(self) -> &'static str {
        match self {
            UniProtField::Accession => "accession",
            UniProtField::Ec => "ec",
            UniProtField::OrganismName => "organism_name",
            UniProtField::ProteinName => "protein_name",
            UniProtField::Sequence => "sequence",
        }
    }
}

/// Field to sort results by
///
/// These are the sortable fields that correspond to the extracted data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum SortField {
    /// Sort by accession number
    Accession,
    /// Sort by EC number
    Ec,
    /// Sort by organism name
    OrganismName,
}

impl SortField {
    /// Convert the sort field enum to its API string representation
    pub(crate) fn to_api_string(self) -> &'static str {
        match self {
            SortField::Accession => "accession",
            SortField::Ec => "ec",
            SortField::OrganismName => "organism_name",
        }
    }
}

/// Sort order direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum SortOrder {
    /// Ascending order
    Asc,
    /// Descending order
    Desc,
}

impl SortOrder {
    /// Convert the sort order enum to its API string representation
    pub(crate) fn to_api_string(self) -> &'static str {
        match self {
            SortOrder::Asc => "asc",
            SortOrder::Desc => "desc",
        }
    }
}

/// Sort configuration combining field and order
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SortOption {
    /// Field to sort by
    pub field: SortField,
    /// Sort order (ascending or descending)
    pub order: SortOrder,
}

impl SortOption {
    /// Convert to API string format (e.g., "accession desc")
    pub(crate) fn to_api_string(self) -> String {
        format!(
            "{} {}",
            self.field.to_api_string(),
            self.order.to_api_string()
        )
    }
}

impl Default for SortOption {
    fn default() -> Self {
        Self {
            field: SortField::Accession,
            order: SortOrder::Desc,
        }
    }
}
