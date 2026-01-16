//! PubChem compound fetching functionality
//!
//! This module provides functions to fetch detailed compound information
//! from the PubChem database using compound names and convert the data
//! into EnzymeML SmallMolecule objects.

use enzymeml::prelude::SmallMolecule;
use urlencoding::encode;

use crate::fetchers::{client::build_client, pubchem::types::PubChemCompound};

/// PubChem REST API endpoint for fetching compound data by name
const PUBCHEM_FETCH_ENDPOINT: &str = "https://pubchem.ncbi.nlm.nih.gov/rest/pug/compound/name/";

/// Fetches compound data from PubChem by name and converts it to a SmallMolecule
///
/// This function queries the PubChem REST API using a compound name to retrieve
/// detailed chemical information including identifiers, properties, and structural
/// data. The response is then converted into an EnzymeML SmallMolecule object.
///
/// # Arguments
/// * `name` - The compound name to search for in the PubChem database
///
/// # Returns
/// * `Ok(SmallMolecule)` - Successfully fetched and converted compound data
/// * `Err(anyhow::Error)` - Network error, API error, parsing failure, or conversion error
///
/// # Errors
/// This function will return an error if:
/// - The HTTP client cannot be built
/// - The network request fails
/// - The PubChem API returns a non-success status code
/// - The response cannot be parsed as valid JSON
/// - The compound data cannot be converted to a SmallMolecule
/// - No compound is found for the given name
pub async fn fetch_pubchem(name: &str) -> anyhow::Result<SmallMolecule> {
    let client = build_client()?;
    let encoded_name = encode(name);
    let url = format!("{PUBCHEM_FETCH_ENDPOINT}/{encoded_name}/JSON");
    let response = client.get(&url).send().await?;
    let mut pubchem_compound: PubChemCompound = response.json().await?;

    // Set the name to the compound name
    pubchem_compound.name = Some(name.to_string());
    pubchem_compound.try_into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_pubchem() {
        let name = "ethanol";
        let small_molecule = fetch_pubchem(name).await.unwrap();

        assert_eq!(small_molecule.name, name);
        assert!(small_molecule.inchi.is_some());
        assert!(small_molecule.inchikey.is_some());
        assert!(small_molecule.canonical_smiles.is_some());
    }
}
