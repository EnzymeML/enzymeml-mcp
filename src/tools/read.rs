//! Tools for reading EnzymeML documents and measurements
//!
//! This module provides tools for reading EnzymeML documents from the Suite application,
//! with options to read the full document structure or just measurements.

use enzymeml::suite;
use toon_format::encode_default;

/// Reads the EnzymeML document structure without measurements
///
/// This function fetches the main EnzymeML document from the Suite desktop application
/// and returns it in TOON (Tree Object Oriented Notation) format. Measurements
/// are excluded to keep the response lightweight.
///
/// # Returns
/// - Success: TOON-encoded EnzymeML document without measurements
/// - Error: Formatted error message describing the failure
///
/// # Note
/// Use `read_measurements` if you need measurement data specifically.
pub async fn read_enzymeml_document(id: impl Into<Option<u64>>) -> String {
    let mut enzymeml_document = match suite::fetch_document_from_suite(id.into(), None) {
        Ok(doc) => doc,
        Err(e) => return format!("Error: Failed to fetch EnzymeML document: {}", e),
    };

    // Remove measurements for lightweight response
    enzymeml_document.measurements.clear();

    match encode_default(&enzymeml_document) {
        Ok(token) => token,
        Err(e) => format!("Error: Failed to encode EnzymeML document: {}", e),
    }
}

/// Reads measurement data from the EnzymeML document
///
/// This function specifically fetches measurement data from the EnzymeML document
/// and returns it in TOON format. This is useful when you only need measurement
/// information without the full document structure.
///
/// # Returns
/// - Success: TOON-encoded measurement data
/// - Error: Formatted error message describing the failure
pub async fn read_measurements(id: impl Into<Option<u64>>) -> String {
    let enzymeml_document = match suite::fetch_document_from_suite(id.into(), None) {
        Ok(doc) => doc,
        Err(e) => return format!("Error: Failed to fetch EnzymeML document: {}", e),
    };

    let measurements = &enzymeml_document.measurements;

    match encode_default(measurements) {
        Ok(token) => token,
        Err(e) => format!("Error: Failed to encode measurements: {}", e),
    }
}

/// Lists all documents from the Suite application
///
/// This function fetches all documents from the Suite desktop application
/// and returns them in TOON format.
///
/// # Returns
/// - Success: TOON-encoded list of documents
/// - Error: Formatted error message describing the failure
pub async fn list_documents() -> String {
    let documents = match suite::list_documents_from_suite(None) {
        Ok(docs) => docs,
        Err(e) => return format!("Error: Failed to list documents: {}", e),
    };

    match encode_default(&documents) {
        Ok(token) => token,
        Err(e) => format!("Error: Failed to encode documents: {}", e),
    }
}
