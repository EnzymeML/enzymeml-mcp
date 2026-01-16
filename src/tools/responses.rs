use crate::tools::plot;
use enzymeml::prelude::EnzymeMLDocument;
use rmcp::model::{Content, IntoContents};

pub struct MultiImageResponse {
    pub text: Option<String>,
    pub images: Vec<String>,
}

impl MultiImageResponse {
    pub fn from_measurement_ids(measurement_ids: Vec<String>, enzmldoc: &EnzymeMLDocument) -> Self {
        let mut images = Vec::new();
        for measurement_id in measurement_ids {
            let image = plot::plot_measurement(&measurement_id, &enzmldoc).unwrap();
            images.push(image);
        }
        Self { text: None, images }
    }

    pub fn from_error(error: String) -> Self {
        Self {
            text: Some(error),
            images: Vec::new(),
        }
    }
}

impl IntoContents for MultiImageResponse {
    fn into_contents(self) -> Vec<Content> {
        if let Some(text) = self.text {
            return vec![Content::text(text)];
        }
        self.images
            .into_iter()
            .map(|image| Content::image(image, "image/png"))
            .collect()
    }
}
