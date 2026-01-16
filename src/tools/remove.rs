//! Tools for removing elements from EnzymeML documents
//!
//! This module provides functionality to selectively remove various components
//! from EnzymeML documents, including vessels, proteins, small molecules,
//! reactions, parameters, and complexes. It supports both complete removal
//! of entire objects and partial removal of specific elements within reactions.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use toon_format::encode_default;

use enzymeml::{
    prelude::{EnzymeMLDocument, Reaction},
    suite,
    validation::consistency,
};

pub fn remove_from_enzymeml_document(
    document_id: Option<u64>,
    removal_details: RemovalDetails,
) -> Result<String, String> {
    let mut enzmldoc = match suite::fetch_document_from_suite(document_id, None) {
        Ok(doc) => doc,
        Err(e) => {
            return Err(format!(
                "Error: Failed to fetch current EnzymeML document: {}",
                e
            ));
        }
    };
    removal_details.remove_from_enzymeml_document(&mut enzmldoc)?;

    // Validate the document for consistency
    let validator = consistency::check_consistency(&enzmldoc);
    if !validator.is_valid {
        let toon_report =
            encode_default(&validator.errors).unwrap_or_else(|e| format!("Error: {}", e));
        return Err(format!("Error: Document is not valid:\n{toon_report}"));
    }

    match suite::push_document_to_suite(&enzmldoc, document_id.map(|id| id.to_string())) {
        Ok(_) => {
            let mut success_message = format!(
                "Elements removed successfully. The document has been validated for consistency and is ready to be used."
            );

            if !validator.errors.is_empty() {
                let toon_report =
                    encode_default(&validator.errors).unwrap_or_else(|e| format!("Error: {}", e));
                success_message += &format!(
                    "\nThere have been inconsistencies in the document. The report is:\n{toon_report}"
                );
            }

            Ok(success_message)
        }
        Err(e) => Err(format!("Error: Failed to push document to Suite: {}", e)),
    }
}

/// Details specifying which elements to remove from an EnzymeML document
///
/// This structure allows for fine-grained control over what gets removed
/// from an EnzymeML document. Each field contains a list of IDs or actions
/// specifying which elements should be removed.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RemovalDetails {
    /// The IDs of the vessels to remove from the document
    #[schemars(description = "The IDs of the vessels to remove")]
    #[serde(default)]
    pub vessels: Vec<String>,

    /// The IDs of the proteins to remove from the document
    #[schemars(description = "The IDs of the proteins to remove")]
    pub proteins: Vec<String>,

    /// The IDs of the small molecules to remove from the document
    #[schemars(description = "The IDs of the small molecules to remove")]
    pub small_molecules: Vec<String>,

    /// Map of reaction IDs to their corresponding removal actions
    ///
    /// This allows for either complete removal of reactions or partial
    /// removal of specific elements within reactions (reactants, products, modifiers)
    #[schemars(description = "The IDs of the reactions to remove")]
    pub reactions: HashMap<String, ReactionRemovalAction>,

    /// The IDs of the parameters to remove from the document
    #[schemars(description = "The IDs of the parameters to remove")]
    pub parameters: Vec<String>,

    /// The IDs of the complexes to remove from the document
    #[schemars(description = "The IDs of the complexes to remove")]
    pub complexes: Vec<String>,
}

/// Action to take when removing elements from a reaction
///
/// This enum allows for either complete removal of a reaction or
/// selective removal of specific elements within the reaction.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum ReactionRemovalAction {
    /// Remove the entire reaction from the document
    #[schemars(description = "The reaction to remove completely")]
    Complete,

    /// Remove only specific elements from the reaction
    ///
    /// This allows for surgical removal of individual reactants,
    /// products, or modifiers while keeping the reaction itself
    #[schemars(description = "The elements of the reaction to remove")]
    Partial(Vec<ReactionElementRemoval>),
}

/// Specification for removing a specific element from a reaction
///
/// This structure identifies which species (reactant, product, or modifier)
/// should be removed from a reaction by its species ID.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReactionElementRemoval {
    /// The ID of the species to remove from the reaction
    ///
    /// This species will be removed from all roles it plays in the reaction
    /// (reactant, product, and/or modifier)
    #[schemars(description = "The ID of the species to remove")]
    pub species_id: String,
}

impl RemovalDetails {
    /// Remove specified elements from an EnzymeML document
    ///
    /// This method applies all the removal specifications in this `RemovalDetails`
    /// instance to the provided EnzymeML document. Elements are removed by
    /// filtering out items whose IDs match those specified for removal.
    ///
    /// # Arguments
    /// * `enzmldoc` - Mutable reference to the EnzymeML document to modify
    ///
    /// # Returns
    /// * `Ok(())` - If all removals were successful
    /// * `Err(String)` - If any removal operation failed
    ///
    /// # Note
    /// Reactions are handled specially - they are not removed here but should
    /// be processed separately using the `reactions` field and `ReactionRemovalAction`.
    pub fn remove_from_enzymeml_document(
        self,
        enzmldoc: &mut EnzymeMLDocument,
    ) -> Result<(), String> {
        // Remove vessels by filtering out those with matching IDs
        enzmldoc
            .vessels
            .retain(|vessel| !self.vessels.contains(&vessel.id));

        // Remove proteins by filtering out those with matching IDs
        enzmldoc
            .proteins
            .retain(|protein| !self.proteins.contains(&protein.id));

        // Remove small molecules by filtering out those with matching IDs
        enzmldoc
            .small_molecules
            .retain(|small_molecule| !self.small_molecules.contains(&small_molecule.id));

        // Remove parameters by filtering out those with matching IDs
        enzmldoc
            .parameters
            .retain(|parameter| !self.parameters.contains(&parameter.id));

        // Remove complexes by filtering out those with matching IDs
        enzmldoc
            .complexes
            .retain(|complex| !self.complexes.contains(&complex.id));

        // Remove reactions either completely or partially
        for (reaction_id, removal_action) in self.reactions {
            if let Some(reaction) = enzmldoc
                .reactions
                .iter_mut()
                .find(|reaction| reaction.id == reaction_id)
            {
                match &removal_action {
                    ReactionRemovalAction::Complete => {
                        enzmldoc
                            .reactions
                            .retain(|reaction| reaction.id != reaction_id);
                    }
                    ReactionRemovalAction::Partial(_) => {
                        removal_action.remove_from_reaction(reaction)?;
                    }
                }
            }
        }

        Ok(())
    }
}

impl ReactionRemovalAction {
    /// Apply the removal action to a specific reaction
    ///
    /// This method processes the removal action on the provided reaction.
    /// For complete removal, the reaction should be removed from the document
    /// entirely (handled by the caller). For partial removal, specific
    /// species are removed from the reaction's reactants, products, and modifiers.
    ///
    /// # Arguments
    /// * `reaction` - Mutable reference to the reaction to modify
    ///
    /// # Returns
    /// * `Ok(())` - If the removal action was applied successfully
    /// * `Err(String)` - If the removal action failed
    ///
    /// # Note
    /// For `Complete` removal, this method returns `Ok(())` but doesn't
    /// actually remove the reaction - the caller is responsible for removing
    /// the entire reaction from the document.
    pub fn remove_from_reaction(self, reaction: &mut Reaction) -> Result<(), String> {
        match self {
            ReactionRemovalAction::Complete => {
                // For complete removal, the reaction should be removed entirely
                // by the caller - nothing to do here
                Ok(())
            }
            ReactionRemovalAction::Partial(elements) => {
                // Remove specified species from all reaction roles
                for element in elements {
                    // Remove from reactants
                    reaction
                        .reactants
                        .retain(|reactant| reactant.species_id != element.species_id);

                    // Remove from products
                    reaction
                        .products
                        .retain(|product| product.species_id != element.species_id);

                    // Remove from modifiers
                    reaction
                        .modifiers
                        .retain(|modifier| modifier.species_id != element.species_id);
                }
                Ok(())
            }
        }
    }
}
