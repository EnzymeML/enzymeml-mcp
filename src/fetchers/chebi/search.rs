// https://www.ebi.ac.uk/chebi/backend/api/public/es_search/?term=ethanol&page=1&size=15

use enzymeml::prelude::SmallMolecule;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::types::ChebiSearchResponse;
use crate::fetchers::client::build_client;

const CHEBI_SEARCH_ENDPOINT: &str = "https://www.ebi.ac.uk/chebi/backend/api/public/es_search/";

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ChebiSearch {
    #[schemars(description = "The term to search for")]
    pub term: String,
    #[schemars(description = "The page number to search on")]
    #[serde(default = "default_page")]
    pub page: usize,
    #[schemars(description = "The number of results per page")]
    #[serde(default = "default_size")]
    pub size: usize,
}

fn default_page() -> usize {
    1
}

fn default_size() -> usize {
    15
}

impl ChebiSearch {
    pub async fn search(&self) -> anyhow::Result<Vec<SmallMolecule>> {
        let client = build_client()?;
        let url = self.build_url();
        let response = client.get(&url).send().await?;

        if !response.status().is_success() {
            let response_status = response.status();
            let error_text = response.text().await?;
            anyhow::bail!(
                "ChEBI search API request failed with status [{response_status}]: {error_text}"
            );
        }

        let chebi_response: ChebiSearchResponse = response.json().await?;
        Ok(chebi_response
            .results
            .into_iter()
            .map(|result| result.source.into())
            .collect::<Vec<SmallMolecule>>())
    }

    pub async fn fetch(accession: String) -> anyhow::Result<SmallMolecule> {
        let query = Self {
            term: accession.clone(),
            page: 1,
            size: 1,
        };

        let results = query.search().await?;
        if results.is_empty() {
            anyhow::bail!("No results found for accession: {}", accession);
        }

        Ok(results[0].clone())
    }

    fn build_url(&self) -> String {
        format!(
            "{}?term={}&page={}&size={}",
            CHEBI_SEARCH_ENDPOINT, self.term, self.page, self.size
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_search() {
        let query = ChebiSearch {
            term: "ethanol".to_string(),
            page: 1,
            size: 15,
        };
        let results = query.search().await.unwrap();
        assert!(!results.is_empty());
    }
}
