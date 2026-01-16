//! PubChem to EnzymeML mapping utilities
//!
//! This module provides functionality to convert PubChem compound data
//! into EnzymeML SmallMolecule objects, extracting relevant chemical
//! properties and identifiers.

use enzymeml::prelude::SmallMolecule;

use crate::fetchers::pubchem::types::{Compound, PCValue, PubChemCompound};

/// Property label for SMILES notation in PubChem data
const SMILES: &str = "SMILES";
/// Property label for InChI string in PubChem data
const INCHI: &str = "InChI";
/// Property label for InChI Key in PubChem data
const INCHIKEY: &str = "InChIKey";

/// Converts a PubChem compound into an EnzymeML SmallMolecule
///
/// This implementation extracts key chemical identifiers and properties
/// from PubChem compound data and maps them to the corresponding fields
/// in an EnzymeML SmallMolecule object.
///
/// # Extracted Properties
/// - **ID**: PubChem Compound ID (CID) as string
/// - **Name**: Compound name from PubChem
/// - **InChI**: International Chemical Identifier string
/// - **InChI Key**: Hashed version of InChI for faster searching
/// - **Canonical SMILES**: Simplified molecular-input line-entry system notation
///
/// # Returns
/// - `Ok(SmallMolecule)`: Successfully converted compound
/// - `Err(anyhow::Error)`: Conversion failed (currently not used but reserved for future validation)
///
/// # Errors
/// This function will return an error if:
/// - The PubChem compound data contains no compounds
/// - The compound name is missing from the data
/// - The compound ID cannot be extracted from the identifier structure
///
/// # Note
/// Synonymous names are currently set to an empty vector as PubChem
/// compound data doesn't include alternative names in the current structure.
impl TryFrom<PubChemCompound> for SmallMolecule {
    type Error = anyhow::Error;
    fn try_from(result: PubChemCompound) -> Result<SmallMolecule, Self::Error> {
        let compound = result
            .pc_compounds
            .first()
            .ok_or(anyhow::anyhow!("No compound found"))?;
        let name = result.name.ok_or(anyhow::anyhow!("No name found"))?;
        let id = format!("PC_{}", compound.id.get_id()?);
        let inchi = str_prop(compound, INCHI);
        let inchikey = str_prop(compound, INCHIKEY);
        let canonical_smiles = str_prop(compound, SMILES);
        let synonymous_names = Vec::new();

        Ok(SmallMolecule {
            id,
            name,
            inchi,
            inchikey,
            canonical_smiles,
            synonymous_names,
            ..Default::default()
        })
    }
}

/// Extracts a string property value from PubChem compound data
///
/// Searches through all properties of a compound to find a property
/// with the specified label and returns its string value if found.
/// This function only returns values that are stored as string types
/// in the PubChem property value enumeration.
///
/// # Arguments
/// - `compound`: Reference to the PubChem compound data containing properties
/// - `property_label`: The property label to search for (case-sensitive)
///
/// # Returns
/// - `Some(String)`: Property value if found and is a string type
/// - `None`: Property not found or not a string value
///
/// # Supported Property Labels
/// Common property labels that can be extracted include:
/// - "SMILES" - Canonical SMILES notation
/// - "InChI" - International Chemical Identifier
/// - "InChIKey" - Hashed InChI key
/// - "Molecular Formula" - Chemical formula
/// - "IUPAC Name" - IUPAC systematic name
/// - "Molecular Weight" - Molecular weight (though this would be numeric)
fn str_prop(compound: &Compound, property_label: &str) -> Option<String> {
    compound.props.iter().find_map(|prop| {
        if prop.urn.label == property_label {
            match &prop.value {
                PCValue::Sval(value) => Some(value.clone()),
                _ => None,
            }
        } else {
            None
        }
    })
}
