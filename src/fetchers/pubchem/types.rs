//! PubChem API data types
//!
//! This module contains data structures for parsing PubChem API responses.
//! It includes types for both autocomplete search results and detailed compound data.
//! Only the fields we actually need are defined - serde will ignore extra fields.

use serde::Deserialize;

/// PubChem autocomplete API response structure
///
/// This is a minimal struct that only parses the compound names from the
/// dictionary_terms field. The status and total fields are ignored since
/// we don't need them for autocomplete functionality.
#[derive(Debug, Deserialize)]
pub struct PubChemACResponse {
    /// Dictionary terms containing compound names
    #[serde(rename = "dictionary_terms")]
    pub dictionary_terms: DictionaryTerms,
}

/// Dictionary terms wrapper
///
/// Contains the compound names returned by the PubChem autocomplete API.
/// This structure wraps the array of compound names in the expected format
/// from the PubChem REST API response.
#[derive(Debug, Deserialize)]
pub struct DictionaryTerms {
    /// Array of compound names from the autocomplete search
    pub compound: Vec<String>,
}

/// PubChem compound data response structure
///
/// This structure represents the response from PubChem's compound data API,
/// containing detailed information about a specific compound including its
/// identifier and associated properties.
#[derive(Debug, Deserialize)]
pub struct PubChemCompound {
    /// Compound name
    #[serde(rename = "name")]
    pub name: Option<String>,

    /// Array of compound data objects containing properties
    #[serde(rename = "PC_Compounds")]
    pub pc_compounds: Vec<Compound>,
}

/// Individual compound data structure
///
/// Represents a single compound with its associated properties.
/// Each compound contains an array of property objects that describe
/// various chemical and physical characteristics.
#[derive(Debug, Deserialize)]
pub struct Compound {
    /// Array of compound properties (name, formula, SMILES, etc.)
    #[serde(rename = "props")]
    pub props: Vec<Property>,

    /// Compound identifier (CID, SID, or AID)
    pub id: RawIdentifier,
}

/// Raw identifier structure for deserialization
///
/// Internal structure used to parse the nested identifier format
/// from PubChem API responses before converting to PCIdentifier.
#[derive(Debug, Deserialize)]
pub struct RawIdentifier {
    /// Nested identifier object
    pub id: RawIdentifierInner,
}

impl RawIdentifier {
    pub fn get_id(&self) -> anyhow::Result<String> {
        self.id.get_id()
    }
}

/// Inner identifier structure
///
/// Contains the actual identifier values. Only one field will be
/// present in any given response, depending on the identifier type.
#[derive(Debug, Deserialize)]
pub struct RawIdentifierInner {
    /// Compound ID (if present)
    #[serde(default)]
    pub cid: Option<u64>,
    /// Substance ID (if present)
    #[serde(default)]
    pub sid: Option<u64>,
    /// Assay ID (if present)
    #[serde(default)]
    pub aid: Option<u64>,
}

impl RawIdentifierInner {
    pub fn get_id(&self) -> anyhow::Result<String> {
        if let Some(cid) = self.cid {
            Ok(cid.to_string())
        } else if let Some(sid) = self.sid {
            Ok(sid.to_string())
        } else if let Some(aid) = self.aid {
            Ok(aid.to_string())
        } else {
            Err(anyhow::anyhow!("No ID found"))
        }
    }
}

/// Compound property structure
///
/// Represents a single property of a compound, containing both
/// the property identifier (urn) and its value. Properties can include
/// molecular formula, SMILES notation, InChI strings, and other chemical data.
#[derive(Debug, Deserialize)]
pub struct Property {
    /// Property identifier and metadata
    #[serde(rename = "urn")]
    pub urn: Urn,
    /// Property value (can be integer, float, string, or boolean)
    pub value: PCValue,
}

/// Property identifier structure
///
/// Contains metadata about a property, including its label
/// and optional name for identification purposes. The label
/// identifies the type of property (e.g., "IUPAC Name", "Molecular Formula").
#[derive(Debug, Deserialize)]
pub struct Urn {
    /// Property label identifier
    pub label: String,
    /// Optional property name for additional context
    pub name: Option<String>,
}

/// PubChem property value enumeration
///
/// Represents the different types of values that can be associated
/// with compound properties in the PubChem database. Values are
/// strongly typed to match the expected data format.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PCValue {
    /// Integer value (e.g., atom count, bond count)
    Ival(i64),
    /// Floating point value (e.g., molecular weight, logP)
    Fval(f64),
    /// String value (e.g., IUPAC name, SMILES, InChI)
    Sval(String),
    /// Boolean value (e.g., chirality flags)
    Bval(bool),
    /// Binary value (e.g., binary data)
    Binary(String),
}
