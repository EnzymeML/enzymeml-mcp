//! EnzymeML MCP Server
//!
//! This is the main entry point for the EnzymeML Model Context Protocol server.
//! It initializes the server and handles communication with the EnzymeML Suite desktop application.

use rmcp::{ServiceExt, transport::stdio};

use crate::server::EnzymeMLSuiteServer;

mod server;
mod fetchers {
    pub mod client;
    pub mod uniprot {
        pub use super::uniprot::fetch::fetch_uniprot;
        pub use super::uniprot::search::ProteinSearch;
        pub use super::uniprot::search::search_uniprot;

        pub mod fetch;
        pub mod mapping;
        pub mod search;
        pub mod types;
        pub mod url;
    }
    pub mod chebi {
        pub use super::chebi::search::ChebiSearch;

        pub mod mapping;
        pub mod search;
        pub mod types;
    }
    pub mod pubchem {
        pub use super::pubchem::fetch::fetch_pubchem;
        pub use super::pubchem::search::PubChemSearch;

        pub mod fetch;
        pub mod mapping;
        pub mod search;
        pub mod types;
    }
}
mod tools {
    pub mod extend;
    pub mod merge;
    pub mod overview;
    pub mod plot;
    pub mod read;
    pub mod remove;
    pub mod responses;
    pub mod types;
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server = EnzymeMLSuiteServer::new();
    let service = server.serve(stdio()).await?;
    service.waiting().await?;

    Ok(())
}
