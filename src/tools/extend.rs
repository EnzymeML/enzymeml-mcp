//! Tools for extending EnzymeML documents
//!
//! This module provides tools for merging new data into existing EnzymeML documents
//! with automatic validation and consistency checking.

use enzymeml::{
    prelude::{EnzymeMLDocument, SmallMolecule},
    suite,
    validation::consistency,
};
use serde_json;

use crate::tools::{merge, types};
use enzymeml::prelude::Protein;

/// Extends the current EnzymeML document with new data
///
/// This function merges new EnzymeML data into the existing document from the Suite
/// application. It performs intelligent merging by:
/// - Replacing existing items with matching IDs
/// - Adding new items that don't exist
/// - Validating the merged document for consistency
///
/// # Arguments
/// * `document_id` - Optional document ID to work with (None for default document)
/// * `new_enzmldoc` - The new EnzymeML document data to merge
///
/// # Returns
/// - Success: Success message indicating successful merge
/// - Error: Detailed validation report if the merge fails consistency checks
///
/// # Validation
/// The merged document is automatically validated for consistency. If validation
/// fails, a detailed JSON report is returned describing the issues.
pub async fn extend_enzymeml_document(
    document_id: Option<u64>,
    new_enzmldoc: types::EnzymeMLDocumentUpdate,
) -> String {

    // Fetch or use the proteins from the new document
    let proteins = match fetch_proteins(new_enzmldoc.proteins).await {
        Ok(proteins) => proteins,
        Err(e) => return format!("Error: {}", e),
    };

    // Fetch or use the small molecules from the new document
    let small_molecules = match fetch_small_molecules(new_enzmldoc.small_molecules).await {
        Ok(small_molecules) => small_molecules,
        Err(e) => return format!("Error: {}", e),
    };

    // Create an intermediate document to build the result
    let new_enzmldoc = EnzymeMLDocument {
        vessels: new_enzmldoc.vessels,
        proteins,
        small_molecules,
        reactions: new_enzmldoc.reactions,
        parameters: new_enzmldoc.parameters,
        complexes: new_enzmldoc.complexes,
        ..Default::default()
    };

    // Fetch the current document from the Suite application
    let mut current_enzmldoc = match suite::fetch_document_from_suite(document_id, None) {
        Ok(doc) => doc,
        Err(e) => return format!("Error: Failed to fetch current EnzymeML document: {}", e),
    };

    // Merge the new document into the current document
    if let Err(e) = merge::merge_enzymeml_documents(&mut current_enzmldoc, &new_enzmldoc) {
        return e;
    }

    // Validate the merged document for consistency
    let validator = consistency::check_consistency(&current_enzmldoc);

    if validator.is_valid {
        let mut success_message = format!(
            "Document extended successfully. The merged document has been validated for consistency and is ready to be used. The document now contains the new data from the updated EnzymeML document."
        );

        if !validator.errors.is_empty() {
            let toon_report = toon_format::encode_default(&validator.errors).unwrap();

            success_message += &format!(
                "\nThere have been inconsistencies in the document. The report is:\n{}",
                toon_report
            );
        }

        match suite::push_document_to_suite(
            &current_enzmldoc,
            document_id.map(|id| id.to_string()),
        ) {
            Ok(_) => success_message,
            Err(e) => format!("Error: Failed to push document to Suite: {}", e),
        }
    } else {
        match serde_json::to_string_pretty(&validator) {
            Ok(json_report) => format!(
                "Error: Document validation failed. Validation report:\n{}",
                json_report
            ),
            Err(e) => format!(
                "Error: Validation failed and could not generate report: {}",
                e
            ),
        }
    }
}

/// Converts protein sources to Protein objects by fetching from UniProt if needed
async fn fetch_proteins(
    protein_sources: Vec<types::ProteinSource>,
) -> Result<Vec<Protein>, String> {
    let protein_futures: Vec<_> = protein_sources
        .into_iter()
        .map(|protein| protein.try_into_protein())
        .collect();

    match futures::future::try_join_all(protein_futures).await {
        Ok(proteins) => Ok(proteins),
        Err(e) => Err(format!("Failed to fetch proteins: {}", e)),
    }
}

async fn fetch_small_molecules(
    small_molecule_sources: Vec<types::SmallMoleculeSource>,
) -> Result<Vec<SmallMolecule>, String> {
    let small_molecule_futures: Vec<_> = small_molecule_sources
        .into_iter()
        .map(|small_molecule| small_molecule.try_into_small_molecule())
        .collect();

    match futures::future::try_join_all(small_molecule_futures).await {
        Ok(small_molecules) => Ok(small_molecules),
        Err(e) => Err(format!("Failed to fetch small molecules: {}", e)),
    }
}
