//! EnzymeML Suite MCP Server
//!
//! This module provides the main server implementation for the EnzymeML MCP server.
//! It handles the server setup and tool routing.
//!
//! # Exposed Tools Overview
//!
//! This server exposes the following tools for interacting with EnzymeML documents:
//!
//! ## Document Reading Tools
//!
//! - **`enzymeml_document_overview`**: Generates a high-level overview of the EnzymeML document
//!   structure and relationships. Returns data in TOON format. Essential for understanding
//!   document structure before performing edits.
//!
//! - **`read_enzymeml_document`**: Reads the complete EnzymeML document structure from the
//!   Suite desktop application, excluding measurements for performance. Returns TOON format.
//!
//! - **`read_measurements`**: Specifically fetches measurement data from the EnzymeML document.
//!   Returns detailed experimental data including time series, concentrations, and measured values
//!   in TOON format.
//!
//! ## Document Modification Tools
//!
//! - **`extend_enzymeml_document`**: Intelligently merges new data into the existing EnzymeML
//!   document. Supports both adding new items and performing surgical edits to existing items
//!   (identified by ID). Performs automatic validation and consistency checks.
//!
//! - **`remove_from_enzymeml_document`**: Selectively removes elements from the EnzymeML document.
//!   Supports complete removal of objects (vessels, proteins, small molecules, reactions,
//!   parameters, complexes) and partial removal within reactions (reactants, products, modifiers).
//!
//! ## External Database Search Tools
//!
//! - **`search_uniprot`**: Searches the UniProt Knowledgebase for proteins. Supports Boolean
//!   operators (AND, OR, NOT) and field-based filtering (accession, EC number, organism,
//!   protein name, sequence, etc.).
//!
//! - **`search_pubchem`**: Searches the PubChem database for small molecules. Supports Boolean
//!   operators and field-based filtering (name, formula, InChI, InChIKey, SMILES, mass, etc.).
//!   Preferred over ChEBI for small molecule searches.
//!
//! - **`search_chebi`**: Searches the ChEBI database for small molecules. Supports Boolean
//!   operators and field-based filtering. Use when PubChem results are insufficient.
//!
//! ## Visualization Tools
//!
//! - **`plot_measurements`**: Generates SVG plots of measurements from the EnzymeML document.
//!   Can plot all measurements or a specified subset by measurement IDs. Returns a list of
//!   images in MultiImageResponse format.

use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    tool, tool_handler, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use toon_format::encode_default;

use crate::{
    fetchers::{self, chebi::ChebiSearch, pubchem::PubChemSearch, uniprot::ProteinSearch},
    tools::{extend, overview::Overview, read, remove, responses::MultiImageResponse, types},
};

/// EnzymeML Suite MCP Server
///
/// This server provides tools for interacting with EnzymeML documents through the
/// Model Context Protocol. It connects to the EnzymeML Suite desktop application
/// to read and manipulate biochemical data.
#[derive(Clone)]
pub struct EnzymeMLSuiteServer {
    /// Router for handling tool requests
    pub tool_router: ToolRouter<Self>,
}

#[tool_handler(router = self.tool_router)]
impl rmcp::ServerHandler for EnzymeMLSuiteServer {}

impl Default for EnzymeMLSuiteServer {
    fn default() -> Self {
        Self::new()
    }
}

#[tool_router(router = tool_router)]
impl EnzymeMLSuiteServer {
    /// Creates a new EnzymeML Suite server instance
    ///
    /// # Returns
    /// A new server with initialized tool router
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(
        name = "how_to_use_the_enzymeml_suite",
        description = "Before performing your first tool call, you should have a look at the instructions to understand the tools and how to use them. Once you have read the instructions, you dont need to call this tool again except for when you want to refresh your memory."
    )]
    pub async fn get_instructions(&self) -> String {
        let instructions = include_str!("instructions.md");
        instructions.to_string()
    }

    #[tool(
        name = "list_documents",
        description = "Instructions via the tool 'how_to_use_the_enzymeml_suite' should have been read at some point before invocation. Lists all documents from the Suite application and returns them in TOON format. This is useful when you want to know which documents are available to you and which one you want to work with, if tasked to query other than the default document."
    )]
    pub async fn list_documents(&self) -> String {
        read::list_documents().await
    }

    /// Generates an overview of the EnzymeML document
    ///
    /// This tool generates an overview of the EnzymeML document and returns it in TOON format.
    #[tool(
        name = "enzymeml_document_overview",
        description = "Instructions via 'how_to_use_the_enzymeml_suite' should have been read at some point before invocation. Generates an overview of the EnzymeML document and returns it in TOON format. You should use this tool to get a rough overview how things are connected in the document. This is particularly important if you plan to perform surgical edits to the document or want to come up with new content for the document to uphold consistency. The document_id parameter should ONLY be used when specifically tasked to query other documents in the database. Otherwise, leave it empty/null to work with the default document."
    )]
    pub async fn enzymeml_document_overview(
        &self,
        params: Parameters<DocumentIdParameter<EmptyStruct>>,
    ) -> String {
        let params = params.0;
        let enzmldoc = match enzymeml::suite::fetch_document_from_suite(params.id, None) {
            Ok(doc) => doc,
            Err(e) => {
                return format!("Error: Failed to fetch EnzymeML document: {}", e);
            }
        };
        let overview = Overview::from(&enzmldoc);
        match encode_default(&overview) {
            Ok(toon) => toon,
            Err(e) => {
                return format!("Error: Failed to encode overview: {}", e);
            }
        }
    }

    /// Reads the EnzymeML document structure without measurements
    ///
    /// This tool fetches the main EnzymeML document from the Suite desktop application
    /// and returns it in TOON (Tree Object Oriented Notation) format. Measurements
    /// are excluded to keep the response lightweight.
    ///
    /// # Returns
    /// - Success: TOON-encoded EnzymeML document without measurements
    /// - Error: Formatted error message describing the failure
    ///
    /// # Note
    /// Use `read_measurements` if you need measurement data specifically.
    #[tool(
        name = "read_enzymeml_document",
        description = "Instructions via the tool 'how_to_use_the_enzymeml_suite' should have been read at some point before invocation. Reads the EnzymeML document from the EnzymeML Suite desktop app and returns it in TOON format. Measurements are excluded for performance. Use 'read_measurements' for measurement data. The document_id parameter should ONLY be used when specifically tasked to query other documents in the database. Otherwise, leave it empty/null to work with the default document."
    )]
    pub async fn read_enzymeml_document(
        &self,
        params: Parameters<DocumentIdParameter<EmptyStruct>>,
    ) -> String {
        let params = params.0;
        read::read_enzymeml_document(params.id).await
    }

    /// Reads measurement data from the EnzymeML document
    ///
    /// This tool specifically fetches measurement data from the EnzymeML document
    /// and returns it in TOON format. This is useful when you only need measurement
    /// information without the full document structure.
    ///
    /// # Returns
    /// - Success: TOON-encoded measurement data
    /// - Error: Formatted error message describing the failure
    #[tool(
        name = "read_measurements",
        description = "Instructions via the tool 'how_to_use_the_enzymeml_suite' should have been read at some point before invocation. Reads measurement data from the EnzymeML document and returns it in TOON format. Use this tool to learn about and analyze the experimental measurements contained in the document, including time series data, concentrations, and other measured values. This provides detailed insight into the experimental data structure and content without the overhead of the full document metadata. The document_id parameter should ONLY be used when specifically tasked to query other documents in the database. Otherwise, leave it empty/null to work with the default document."
    )]
    pub async fn read_measurements(
        &self,
        params: Parameters<DocumentIdParameter<EmptyStruct>>,
    ) -> String {
        let params = params.0;
        read::read_measurements(params.id).await
    }

    /// Extends the current EnzymeML document with new data
    ///
    /// This tool merges new EnzymeML data into the existing document from the Suite
    /// application. It performs intelligent merging by:
    /// - Replacing existing items with matching IDs
    /// - Adding new items that don't exist
    /// - Validating the merged document for consistency
    ///
    /// # Arguments
    /// * `new_enzmldoc` - The new EnzymeML document data to merge
    ///
    /// # Returns
    /// - Success: Success message indicating successful merge
    /// - Error: Detailed validation report if the merge fails consistency checks
    ///
    /// # Validation
    /// The merged document is automatically validated for consistency. If validation
    /// fails, a detailed JSON report is returned describing the issues.
    #[tool(
        name = "extend_enzymeml_document",
        description = "Instructions via the tool 'how_to_use_the_enzymeml_suite' should have been read at some point before invocation. IMPORTANT: Before submitting extension/updates, always ask for confirmation first. If you plan multiple edits, present all edits to teh user, then proceed incrementally. Extends the current EnzymeML document with new data by intelligently merging collections. Supports both adding new items and performing surgical edits to existing items.\n\n**Adding New Items:** Include complete objects with all required fields. For best results, add data incrementally: first add all proteins (search UniProt for metadata), then small molecules (search ChEBI for metadata), then reactions (search Rhea for metadata), and finally measurements. Always search external databases (UniProt, ChEBI, Rhea) first to enrich your data with standardized metadata before adding to the document.\n\n**Surgical Edits to Existing Items:** CRITICAL: Before performing any surgical edits, you MUST first read the EnzymeML document using `enzymeml_document_overview` to discover ALL existing IDs. Surgical edits are only possible when you know the exact ID of the item you want to modify. Without the correct ID, the system cannot identify which existing object to update.\n\nOnce you have the IDs, you can perform incremental, surgical edits on objects that already exist in the document (identified by matching ID):\n- **Optional fields (Option<T>):** Leave fields as `null`/`None` to preserve the existing value unchanged. Only provide a value if you want to update that specific field.\n- **Mandatory string fields:** Use an empty string `\"\"` to preserve the existing value unchanged. Provide a non-empty string only if you want to update that field.\n- **Vector/array fields:** Leave as empty array `[]` to preserve the existing value unchanged. Provide a non-empty array only if you want to replace the entire collection.\n- **Special protected fields:** The `id` field is NEVER overwritable and will always be preserved from the existing object. For Protein objects, the `sequence` field is also protected and will never be overwritten to prevent hallucinations.\n\n**Example surgical edit workflow:** 1) First call `enzymeml_document_overview` to find existing protein IDs, 2) Then to update only the `name` field of an existing protein with ID \"P12345\", provide: `{\"id\": \"P12345\", \"name\": \"New Name\", \"sequence\": null, \"organism\": null, ...}` - all null/empty fields will preserve their existing values, only `name` will be updated.\n\n**Important:** Always perform edits incrementally - make one focused change at a time rather than attempting to update multiple unrelated objects simultaneously."
    )]
    pub async fn extend_enzymeml_document(
        &self,
        params: Parameters<DocumentIdParameter<types::EnzymeMLDocumentUpdate>>,
    ) -> String {
        let params = params.0;
        extend::extend_enzymeml_document(params.id, params.value).await
    }

    /// Removes elements from the current EnzymeML document
    ///
    /// This tool allows you to selectively remove various components from the current
    /// EnzymeML document. It supports complete removal of entire objects (vessels,
    /// proteins, small molecules, reactions, parameters, and complexes) as well as
    /// partial removal of specific elements within reactions (reactants, products, modifiers).
    #[tool(
        name = "remove_from_enzymeml_document",
        description = "Instructions via the tool 'how_to_use_the_enzymeml_suite' should have been read at some point before invocation. IMPORTANT: Before removing anything, always read the EnzymeML document first using `enzymeml_document_overview` to discover ALL existing IDs and understand the document structure. Removes elements from the current EnzymeML document. Supports complete removal of entire objects (vessels, proteins, small molecules, reactions, parameters, and complexes) as well as partial removal of specific elements within reactions (reactants, products, modifiers). You MUST verify that the IDs you want to remove actually exist in the document before attempting removal."
    )]
    pub async fn remove_elements_from_enzymeml_document(
        &self,
        params: Parameters<DocumentIdParameter<remove::RemovalDetails>>,
    ) -> String {
        let params = params.0;
        match remove::remove_from_enzymeml_document(params.id, params.value) {
            Ok(success_message) => success_message,
            Err(error_message) => error_message,
        }
    }

    /// Searches the UniProt database for proteins matching the given query
    ///
    /// This tool performs a search against the UniProt Knowledgebase (UniProtKB) using
    /// the UniProt REST API. The query supports complex Boolean expressions using AND,
    /// OR, and NOT operators to create expressive searches.
    #[tool(
        name = "search_uniprot",
        description = "Instructions via the tool 'how_to_use_the_enzymeml_suite' should have been read at some point before invocation. Searches the UniProt database for proteins matching the given query. Supports Boolean operators (AND, OR, NOT) and field-based filtering (e.g., 'ec:1.1.1.1 AND organism_name:human'). Filterable fields include: accession, ec, organism_name, protein_name, and sequence. See https://rest.uniprot.org/configure/uniprotkb/search-fields for the complete list of searchable fields."
    )]
    pub async fn search_uniprot(&self, query: Parameters<ProteinSearch>) -> String {
        let search = query.0;
        let results = fetchers::uniprot::search_uniprot(search).await;
        results.unwrap_or_else(|e| format!("Error: {}", e))
    }

    /// Searches the ChEBI database for small molecules matching the given query
    ///
    /// This tool performs a search against the ChEBI database using the ChEBI REST API.
    /// The query supports complex Boolean expressions using AND, OR, and NOT operators
    /// to create expressive searches.
    #[tool(
        name = "search_chebi",
        description = "Instructions via the tool 'how_to_use_the_enzymeml_suite' should have been read at some point before invocation. IMPORTANT: Always prefer PubChem over CheBI. If PubChem results are not useful, then use ChEBI.Searches the ChEBI database for small molecules matching the given query. Supports Boolean operators (AND, OR, NOT) and field-based filtering (e.g., 'name:ethanol AND formula:C2H6O'). Filterable fields include: name, formula, inchi, inchikey, smiles, and mass. See https://www.ebi.ac.uk/chebi/backend/api/public/es_search/ for the complete list of searchable fields."
    )]
    pub async fn search_chebi(&self, query: Parameters<ChebiSearch>) -> String {
        let search = query.0;
        let results = search.search().await;

        match results {
            Ok(results) => encode_default(&results).unwrap_or_else(|e| format!("Error: {}", e)),
            Err(e) => format!("Error: {}", e),
        }
    }

    /// Searches the PubChem database for small molecules matching the given query
    ///
    /// This tool performs a search against the PubChem database using the PubChem REST API.
    /// The query supports complex Boolean expressions using AND, OR, and NOT operators
    /// to create expressive searches.
    #[tool(
        name = "search_pubchem",
        description = "Instructions via the tool 'how_to_use_the_enzymeml_suite' should have been read at some point before invocation. Searches the PubChem database for small molecules matching the given query. Supports Boolean operators (AND, OR, NOT) and field-based filtering (e.g., 'name:ethanol AND formula:C2H6O'). Filterable fields include: name, formula, inchi, inchikey, smiles, and mass. See https://pubchem.ncbi.nlm.nih.gov/rest/autocomplete/compound/ for the complete list of searchable fields."
    )]
    pub async fn search_pubchem(&self, query: Parameters<PubChemSearch>) -> String {
        let search = query.0;
        let results = search.search().await;

        match results {
            Ok(results) => results.join("\n"),
            Err(e) => format!("Error: {}", e),
        }
    }

    /// Plots the measurements from the EnzymeML document
    ///
    /// This tool plots the measurements from the EnzymeML document and returns an SVG image.
    #[tool(
        name = "plot_measurements",
        description = "Instructions via the tool 'how_to_use_the_enzymeml_suite' should have been read at some point before invocation. Plots the measurements from the EnzymeML document and returns a list of images. If no measurement ids are provided, all measurements are plotted. The document_id parameter should ONLY be used when specifically tasked to query other documents in the database. Otherwise, leave it empty/null to work with the default document. Important: If you plan to plot a subset of the measurements, you MUST first "
    )]
    pub async fn plot_measurements(
        &self,
        params: Parameters<DocumentIdParameter<ArrayParameter<String>>>,
    ) -> MultiImageResponse {
        let params = params.0;
        let enzmldoc = match enzymeml::suite::fetch_document_from_suite(params.id, None) {
            Ok(doc) => doc,
            Err(e) => {
                return MultiImageResponse::from_error(format!(
                    "Error: Failed to fetch EnzymeML document: {}",
                    e
                ));
            }
        };

        let mut measurement_ids = params.value.value;
        if measurement_ids.is_empty() {
            measurement_ids = enzmldoc.measurements.iter().map(|m| m.id.clone()).collect();
        }

        // Validate that the measurement ids exist in the document
        let mut invalid_measurement_ids = Vec::new();
        for measurement_id in measurement_ids.iter() {
            if !enzmldoc
                .measurements
                .iter()
                .any(|m| m.id == measurement_id.as_str())
            {
                invalid_measurement_ids.push(measurement_id);
            }
        }

        // If there are invalid measurement ids, return an error
        if !invalid_measurement_ids.is_empty() {
            return MultiImageResponse {
                text: Some(format!(
                    "Error: Measurement IDs {} not found",
                    invalid_measurement_ids
                        .iter()
                        .map(|id| id.as_str())
                        .collect::<Vec<&str>>()
                        .join(", ")
                )),
                images: Vec::new(),
            };
        }

        MultiImageResponse::from_measurement_ids(measurement_ids.clone(), &enzmldoc)
    }
}

/// Helper structs for array parameters

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ArrayParameter<T> {
    pub value: Vec<T>,
}

/// Empty struct for functions that only need document_id
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct EmptyStruct {}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DocumentIdParameter<T> {
    #[schemars(
        description = "The ID of the document to work with. This is only used when specifically tasked to query other documents in the database. Otherwise, leave it empty/null to work with the default document."
    )]
    pub id: Option<u64>,
    #[schemars(
        description = "The ID of the document to work with. This is only used when specifically tasked to query other documents in the database. Otherwise, leave it empty/null to work with the default document."
    )]
    #[serde(flatten)]
    pub value: T,
}
