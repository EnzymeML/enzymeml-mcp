//! Jupyter template management for EnzymeML Suite
//!
//! This module provides functionality to interact with Jupyter notebook templates
//! from the EnzymeML Suite desktop application. Templates can be listed and retrieved
//! for use in data analysis workflows.

use reqwest::Client;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The default endpoint for accessing Jupyter templates from the EnzymeML Suite
const SUITE_TEMPLATE_ENDPOINT: &str = "http://127.0.0.1:13452/jupyter/templates";

#[cfg(test)]
thread_local! {
    static TEST_ENDPOINT: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}

/// Gets the appropriate endpoint URL for template requests
///
/// In test mode, returns the test endpoint if set, otherwise falls back to the default.
/// In production mode, always returns the default Suite endpoint.
///
/// # Returns
///
/// The endpoint URL as a String
fn get_endpoint() -> String {
    #[cfg(test)]
    {
        TEST_ENDPOINT.with(|endpoint| {
            endpoint
                .borrow()
                .clone()
                .unwrap_or_else(|| SUITE_TEMPLATE_ENDPOINT.to_string())
        })
    }
    #[cfg(not(test))]
    {
        SUITE_TEMPLATE_ENDPOINT.to_string()
    }
}

/// Request structure for retrieving a Jupyter template
///
/// This struct is used to specify the name of the template to retrieve.
/// You should use the tool 'list_jupyter_templates' to get the list of available templates and their names.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct TemplateRequest {
    #[schemars(
        description = "The ID of the template to retrieve. You should use the tool 'list_jupyter_templates' to get the list of available templates and their IDs."
    )]
    pub id: String,
}

/// Represents a Jupyter notebook template from the EnzymeML Suite
///
/// Templates are pre-configured Jupyter notebooks that provide standardized
/// workflows for common EnzymeML data analysis tasks.
#[derive(Debug, Serialize, Deserialize)]
pub struct JupyterTemplate {
    /// Unique identifier for the template
    id: String,
    /// Human-readable name of the template
    name: String,
    /// Description of what the template does
    description: String,
    /// File system path to the template
    template_path: String,
    /// Repository where the template is stored
    repository: String,
    /// Category classification for the template
    category: String,
}

impl JupyterTemplate {
    /// Lists all available Jupyter templates from the EnzymeML Suite
    ///
    /// This function fetches all available Jupyter notebook templates from the
    /// EnzymeML Suite desktop application.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<JupyterTemplate>)` - A vector of all available templates
    /// * `Err(anyhow::Error)` - If the request fails or the response cannot be parsed
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The Suite API is not accessible
    /// - The HTTP request fails
    /// - The response cannot be parsed as JSON
    pub async fn list() -> anyhow::Result<Vec<JupyterTemplate>> {
        let endpoint = get_endpoint();
        let response = Client::new().get(&endpoint).send().await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to list templates: HTTP {}", response.status());
        }

        let templates: Vec<JupyterTemplate> = response.json().await?;
        Ok(templates)
    }

    /// Retrieves the content of a specific Jupyter template by ID
    ///
    /// This function fetches the actual notebook content for a given template ID.
    /// The template must exist in the Suite's template collection.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier of the template to retrieve
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The template content as a string
    /// * `Err(anyhow::Error)` - If the template doesn't exist or the request fails
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The template with the given ID doesn't exist
    /// - The Suite API is not accessible
    /// - The HTTP request fails
    /// - The response cannot be read as text
    pub async fn get(id: &str) -> anyhow::Result<String> {
        if !Self::exists(id).await? {
            return Err(anyhow::anyhow!("Template with id {} not found", id));
        }

        let endpoint = get_endpoint();
        let response = Client::new()
            .get(format!("{}/{}", &endpoint, id))
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to get template {}: HTTP {}", id, response.status());
        }

        let code = response.text().await?;
        Ok(code.to_string())
    }

    /// Checks if a template with the given ID exists
    ///
    /// This is a helper function that verifies whether a template with the
    /// specified ID is available in the Suite's template collection.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier to check for
    ///
    /// # Returns
    ///
    /// * `Ok(bool)` - `true` if the template exists, `false` otherwise
    /// * `Err(anyhow::Error)` - If the template list cannot be retrieved
    async fn exists(id: &str) -> anyhow::Result<bool> {
        let templates = Self::list().await?;
        Ok(templates.iter().any(|template| template.id == id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    /// Sets up a mock server and returns its base URL
    ///
    /// Creates a new WireMock server instance for testing HTTP interactions.
    /// The server will be automatically cleaned up when the test completes.
    ///
    /// # Returns
    ///
    /// A MockServer instance ready for use in tests
    async fn setup_mock_server() -> MockServer {
        MockServer::start().await
    }

    /// Sets the test endpoint for the current test thread
    ///
    /// This function configures the thread-local test endpoint that will be used
    /// instead of the default Suite endpoint during testing.
    ///
    /// # Arguments
    ///
    /// * `endpoint` - The mock server endpoint URL to use for testing
    fn set_test_endpoint(endpoint: String) {
        TEST_ENDPOINT.with(|e| *e.borrow_mut() = Some(endpoint));
    }

    /// Clears the test endpoint after a test completes
    ///
    /// This function resets the thread-local test endpoint to None, ensuring
    /// that subsequent tests start with a clean state.
    fn clear_test_endpoint() {
        TEST_ENDPOINT.with(|e| *e.borrow_mut() = None);
    }

    /// Creates sample template data for testing
    ///
    /// Generates a vector of sample JupyterTemplate instances that can be used
    /// in various test scenarios. The templates have different IDs, names, and
    /// categories to test different aspects of the functionality.
    ///
    /// # Returns
    ///
    /// A vector containing two sample JupyterTemplate instances
    fn create_sample_templates() -> Vec<JupyterTemplate> {
        vec![
            JupyterTemplate {
                id: "template-1".to_string(),
                name: "Basic Analysis".to_string(),
                description: "A basic analysis template".to_string(),
                template_path: "/path/to/template1.ipynb".to_string(),
                repository: "repo1".to_string(),
                category: "analysis".to_string(),
            },
            JupyterTemplate {
                id: "template-2".to_string(),
                name: "Advanced Plotting".to_string(),
                description: "Advanced plotting template".to_string(),
                template_path: "/path/to/template2.ipynb".to_string(),
                repository: "repo2".to_string(),
                category: "visualization".to_string(),
            },
        ]
    }

    /// Test listing all templates successfully
    ///
    /// Verifies that the list() function correctly retrieves and parses
    /// template data from the Suite API when the server responds successfully.
    #[tokio::test]
    async fn test_list_templates_success() {
        let mock_server = setup_mock_server().await;
        let templates = create_sample_templates();
        let templates_json = serde_json::to_string(&templates).unwrap();

        Mock::given(method("GET"))
            .and(path("/jupyter/templates"))
            .respond_with(ResponseTemplate::new(200).set_body_string(templates_json))
            .mount(&mock_server)
            .await;

        set_test_endpoint(format!("{}/jupyter/templates", mock_server.uri()));

        let result = JupyterTemplate::list().await;
        assert!(result.is_ok());

        let retrieved_templates = result.unwrap();
        assert_eq!(retrieved_templates.len(), 2);
        assert_eq!(retrieved_templates[0].id, "template-1");
        assert_eq!(retrieved_templates[1].id, "template-2");

        clear_test_endpoint();
    }

    /// Test listing templates when server returns empty list
    ///
    /// Verifies that the list() function handles empty responses correctly
    /// and returns an empty vector rather than an error.
    #[tokio::test]
    async fn test_list_templates_empty() {
        let mock_server = setup_mock_server().await;

        Mock::given(method("GET"))
            .and(path("/jupyter/templates"))
            .respond_with(ResponseTemplate::new(200).set_body_string("[]"))
            .mount(&mock_server)
            .await;

        set_test_endpoint(format!("{}/jupyter/templates", mock_server.uri()));

        let result = JupyterTemplate::list().await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());

        clear_test_endpoint();
    }

    /// Test listing templates when server returns error
    ///
    /// Verifies that the list() function properly handles server errors
    /// and returns an appropriate error result.
    #[tokio::test]
    async fn test_list_templates_server_error() {
        let mock_server = setup_mock_server().await;

        Mock::given(method("GET"))
            .and(path("/jupyter/templates"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        set_test_endpoint(format!("{}/jupyter/templates", mock_server.uri()));

        let result = JupyterTemplate::list().await;
        assert!(result.is_err());

        clear_test_endpoint();
    }

    /// Test getting a template by ID successfully
    ///
    /// Verifies that the get() function correctly retrieves template content
    /// when the template exists and the server responds successfully.
    #[tokio::test]
    async fn test_get_template_success() {
        let mock_server = setup_mock_server().await;
        let templates = create_sample_templates();
        let templates_json = serde_json::to_string(&templates).unwrap();
        let template_content = r#"{"cells":[],"metadata":{},"nbformat":4,"nbformat_minor":2}"#;

        // Mock list endpoint for exists() check
        Mock::given(method("GET"))
            .and(path("/jupyter/templates"))
            .respond_with(ResponseTemplate::new(200).set_body_string(templates_json))
            .mount(&mock_server)
            .await;

        // Mock get endpoint
        Mock::given(method("GET"))
            .and(path("/jupyter/templates/template-1"))
            .respond_with(ResponseTemplate::new(200).set_body_string(template_content))
            .mount(&mock_server)
            .await;

        set_test_endpoint(format!("{}/jupyter/templates", mock_server.uri()));

        let result = JupyterTemplate::get("template-1").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), template_content);

        clear_test_endpoint();
    }

    /// Test getting a template that doesn't exist
    ///
    /// Verifies that the get() function returns an appropriate error
    /// when attempting to retrieve a template that doesn't exist.
    #[tokio::test]
    async fn test_get_template_not_found() {
        let mock_server = setup_mock_server().await;
        let templates = create_sample_templates();
        let templates_json = serde_json::to_string(&templates).unwrap();

        Mock::given(method("GET"))
            .and(path("/jupyter/templates"))
            .respond_with(ResponseTemplate::new(200).set_body_string(templates_json))
            .mount(&mock_server)
            .await;

        set_test_endpoint(format!("{}/jupyter/templates", mock_server.uri()));

        let result = JupyterTemplate::get("non-existent-template").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));

        clear_test_endpoint();
    }

    /// Test getting a template when server returns error
    ///
    /// Verifies that the get() function properly handles server errors
    /// even when the template exists (passes the exists() check).
    #[tokio::test]
    async fn test_get_template_server_error() {
        let mock_server = setup_mock_server().await;
        let templates = create_sample_templates();
        let templates_json = serde_json::to_string(&templates).unwrap();

        // Mock list endpoint for exists() check
        Mock::given(method("GET"))
            .and(path("/jupyter/templates"))
            .respond_with(ResponseTemplate::new(200).set_body_string(templates_json))
            .mount(&mock_server)
            .await;

        // Mock get endpoint with error
        Mock::given(method("GET"))
            .and(path("/jupyter/templates/template-1"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        set_test_endpoint(format!("{}/jupyter/templates", mock_server.uri()));

        let result = JupyterTemplate::get("template-1").await;
        assert!(result.is_err());

        clear_test_endpoint();
    }

    /// Test checking if a template exists (true case)
    ///
    /// Tests the exists() logic indirectly by verifying that templates
    /// returned from list() contain the expected template ID.
    #[tokio::test]
    async fn test_exists_template_true() {
        let mock_server = setup_mock_server().await;
        let templates = create_sample_templates();
        let templates_json = serde_json::to_string(&templates).unwrap();

        Mock::given(method("GET"))
            .and(path("/jupyter/templates"))
            .respond_with(ResponseTemplate::new(200).set_body_string(templates_json))
            .mount(&mock_server)
            .await;

        set_test_endpoint(format!("{}/jupyter/templates", mock_server.uri()));

        let list_result = JupyterTemplate::list().await.unwrap();
        assert!(list_result.iter().any(|t| t.id == "template-1"));

        clear_test_endpoint();
    }

    /// Test checking if a template exists (false case)
    ///
    /// Verifies that the exists() logic correctly identifies when a
    /// template ID is not present in the available templates list.
    #[tokio::test]
    async fn test_exists_template_false() {
        let mock_server = setup_mock_server().await;
        let templates = create_sample_templates();
        let templates_json = serde_json::to_string(&templates).unwrap();

        Mock::given(method("GET"))
            .and(path("/jupyter/templates"))
            .respond_with(ResponseTemplate::new(200).set_body_string(templates_json))
            .mount(&mock_server)
            .await;

        set_test_endpoint(format!("{}/jupyter/templates", mock_server.uri()));

        let list_result = JupyterTemplate::list().await.unwrap();
        assert!(!list_result.iter().any(|t| t.id == "non-existent-template"));

        clear_test_endpoint();
    }
}
