//! Shared types for EnzymeML MCP tools

use enzymeml::prelude::{Complex, Parameter, Protein, Reaction, SmallMolecule, Vessel};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::fetchers::{chebi::ChebiSearch, pubchem, uniprot::fetch_uniprot};

/// Update structure for extending EnzymeML documents
///
/// This struct represents a partial EnzymeML document update that can be merged
/// into an existing document. All fields are optional and default to empty vectors.
/// You can add new items or perform surgical edits to existing items by providing
/// their IDs. Empty vectors will be ignored, allowing you to update only specific
/// collections.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[schemars(
    description = "Structure for updating an EnzymeML document. All fields are optional and default to empty vectors. Provide items to add or update (items with existing IDs will be updated, new IDs will be added)."
)]
pub struct EnzymeMLDocumentUpdate {
    #[schemars(
        description = "Vessels to add or update in the document. Items with existing IDs will be updated, new IDs will be added."
    )]
    #[serde(default)]
    pub vessels: Vec<Vessel>,
    #[schemars(
        description = "Proteins to add or update in the document. Can be UniProt accession numbers or direct protein objects. Items with existing IDs will be updated, new IDs will be added."
    )]
    #[serde(default)]
    pub proteins: Vec<ProteinSource>,
    #[schemars(
        description = "Small molecules to add or update in the document. Can be ChEBI IDs, PubChem names, or direct small molecule objects. Items with existing IDs will be updated, new IDs will be added."
    )]
    #[serde(default)]
    pub small_molecules: Vec<SmallMoleculeSource>,
    #[schemars(
        description = "Reactions to add or update in the document. Items with existing IDs will be updated, new IDs will be added."
    )]
    #[serde(default)]
    pub reactions: Vec<Reaction>,
    #[schemars(
        description = "Parameters to add or update in the document. Items with existing IDs will be updated, new IDs will be added."
    )]
    #[serde(default)]
    pub parameters: Vec<Parameter>,
    #[schemars(
        description = "Complexes to add or update in the document. Items with existing IDs will be updated, new IDs will be added."
    )]
    #[serde(default)]
    pub complexes: Vec<Complex>,
}

/// Source for a protein in an EnzymeML document
///
/// Proteins can be specified either by referencing an external database
/// (UniProt) or by providing a complete protein object directly.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[schemars(
    description = "Source for a protein. Can be a UniProt accession number (fetched automatically) or a direct protein object."
)]
pub enum ProteinSource {
    /// UniProt accession number
    #[schemars(
        with = "String",
        description = "The UniProt accession number of the protein. The protein data will be automatically fetched from UniProt."
    )]
    UniProt(String),
    /// Local protein object
    #[schemars(
        description = "A complete protein object provided directly. Use this when you have all the protein information or when the protein is not in UniProt."
    )]
    Direct(Protein),
}

impl ProteinSource {
    pub async fn try_into_protein(self) -> anyhow::Result<Protein> {
        match self {
            ProteinSource::UniProt(accession) => fetch_uniprot(accession).await,
            ProteinSource::Direct(protein) => Ok(protein),
        }
    }
}

/// Source for a small molecule in an EnzymeML document
///
/// Small molecules can be specified by referencing external databases
/// (ChEBI or PubChem) or by providing a complete small molecule object directly.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[schemars(
    description = "Source for a small molecule. Can be a ChEBI ID, PubChem name (fetched automatically), or a direct small molecule object."
)]
pub enum SmallMoleculeSource {
    /// ChEBI accession number
    #[schemars(
        with = "String",
        description = "The ChEBI accession number of the small molecule. The molecule data will be automatically fetched from ChEBI."
    )]
    ChEBI(String),
    /// PubChem name
    #[schemars(
        with = "String",
        description = "The PubChem name of the small molecule. The molecule data will be automatically fetched from PubChem."
    )]
    PubChem(String),
    /// Local small molecule object
    #[schemars(
        description = "A complete small molecule object provided directly. Use this when you have all the molecule information or when the molecule is not in ChEBI or PubChem."
    )]
    Direct(SmallMolecule),
}

impl SmallMoleculeSource {
    pub async fn try_into_small_molecule(self) -> anyhow::Result<SmallMolecule> {
        match self {
            SmallMoleculeSource::ChEBI(accession) => ChebiSearch::fetch(accession).await,
            SmallMoleculeSource::PubChem(name) => pubchem::fetch_pubchem(&name).await,
            SmallMoleculeSource::Direct(small_molecule) => Ok(small_molecule),
        }
    }
}
