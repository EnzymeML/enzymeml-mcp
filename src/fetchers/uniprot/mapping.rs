//! Mapping UniProt API responses to EnzymeML types
//!
//! This module handles converting UniProt API response structures into
//! EnzymeML Protein objects.

use crate::fetchers::uniprot::types::UniProtEntry;
use enzymeml::prelude::Protein;

impl From<UniProtEntry> for Protein {
    fn from(entry: UniProtEntry) -> Self {
        // Extract name from protein description
        let name = entry
            .protein_description
            .as_ref()
            .and_then(|desc| desc.recommended_name.as_ref())
            .map(|rec| rec.full_name.value.clone())
            .unwrap_or_else(|| entry.primary_accession.clone());

        // Extract sequence
        let sequence = entry.sequence.as_ref().map(|seq| seq.value.clone());

        // Extract organism information
        let organism = entry
            .organism
            .as_ref()
            .map(|org| org.scientific_name.clone());

        let organism_tax_id = entry.organism.as_ref().map(|org| org.taxon_id.to_string());

        // Extract EC number from recommended name (join multiple EC numbers with commas)
        let ecnumber = entry
            .protein_description
            .as_ref()
            .and_then(|desc| desc.recommended_name.as_ref())
            .and_then(|rec| rec.ec_numbers.as_ref())
            .map(|ecs| {
                ecs.iter()
                    .map(|ec| ec.value.clone())
                    .collect::<Vec<_>>()
                    .join(",")
            });

        Protein {
            id: entry.primary_accession.clone(),
            name,
            constant: true,
            sequence,
            vessel_id: None,
            ecnumber,
            organism,
            organism_tax_id,
            references: Vec::new(),
        }
    }
}
