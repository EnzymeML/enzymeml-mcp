//! Mapping ChEBI API responses to EnzymeML types
//!
//! This module handles converting ChEBI API response structures into
//! EnzymeML SmallMolecule objects.

use crate::fetchers::chebi::types::ChebiCompound;
use enzymeml::prelude::SmallMolecule;

impl From<ChebiCompound> for SmallMolecule {
    fn from(compound: ChebiCompound) -> Self {
        // Use ASCII name as a synonymous name if it differs from the primary name
        let mut synonymous_names = Vec::new();
        if compound.ascii_name != compound.name {
            synonymous_names.push(compound.ascii_name);
        }

        // Add ChEBI accession as a reference
        let references = vec![compound.chebi_accession.clone()];

        SmallMolecule {
            id: compound.chebi_accession,
            name: compound.name,
            constant: false,
            vessel_id: None,
            canonical_smiles: compound.smiles.clone(),
            inchi: compound.inchi.clone(),
            inchikey: compound.inchikey.clone(),
            synonymous_names,
            references,
        }
    }
}
