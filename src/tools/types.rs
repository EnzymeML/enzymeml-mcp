//! Shared types for EnzymeML MCP tools

use enzymeml::prelude::{Complex, Parameter, Protein, Reaction, SmallMolecule, Vessel};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::fetchers::{chebi::ChebiSearch, pubchem, uniprot::fetch_uniprot};

/// Update structure for extending EnzymeML documents
///
/// This struct represents a partial EnzymeML document update that can be merged
/// into an existing document. All fields are optional and default to empty vectors.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct EnzymeMLDocumentUpdate {
    #[serde(default)]
    pub vessels: Vec<Vessel>,
    #[serde(default)]
    pub proteins: Vec<ProteinSource>,
    #[serde(default)]
    pub small_molecules: Vec<SmallMoleculeSource>,
    #[serde(default)]
    pub reactions: Vec<Reaction>,
    #[serde(default)]
    pub parameters: Vec<Parameter>,
    #[serde(default)]
    pub complexes: Vec<Complex>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub enum ProteinSource {
    /// UniProt accession number
    #[schemars(
        with = "String",
        description = "The UniProt accession number of the protein"
    )]
    UniProt(String),
    /// Local protein object
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

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub enum SmallMoleculeSource {
    /// ChEBI accession number
    #[schemars(
        with = "String",
        description = "The ChEBI accession number of the small molecule"
    )]
    ChEBI(String),
    /// PubChem name
    #[schemars(
        with = "String",
        description = "The PubChem name of the small molecule"
    )]
    PubChem(String),
    /// Local small molecule object
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
