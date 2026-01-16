//! Merging logic for EnzymeML documents
//!
//! This module provides utilities for intelligently merging EnzymeML documents,
//! including field-level merging and consistency checks.

use enzymeml::prelude::{
    Complex, EnzymeMLDocument, Parameter, Protein, Reaction, SmallMolecule, Vessel,
};

/// Macro to merge an Option field: uses other's value if Some, otherwise keeps self's value.
///
/// # Example
/// ```ignore
/// let vessel_id = merge_option!(other.vessel_id, self.vessel_id);
/// ```
macro_rules! merge_option {
    ($other:expr, $self:expr) => {
        $other.clone().or($self.clone())
    };
}

/// Macro to merge an Option field without cloning: uses other's value if Some, otherwise keeps self's value.
///
/// # Example
/// ```ignore
/// let value = merge_option_direct!(other.value, self.value);
/// ```
macro_rules! merge_option_direct {
    ($other:expr, $self:expr) => {
        $other.or($self)
    };
}

/// Macro to merge a String field: uses other's value if not empty, otherwise keeps self's value.
///
/// # Example
/// ```ignore
/// let name = merge_string!(other.name, self.name);
/// ```
macro_rules! merge_string {
    ($other:expr, $self:expr) => {
        if $other.is_empty() {
            $self.clone()
        } else {
            $other.clone()
        }
    };
}

/// Macro to merge a Vec field: uses other's value if not empty, otherwise keeps self's value.
///
/// # Example
/// ```ignore
/// let references = merge_vec!(other.references, self.references);
/// ```
macro_rules! merge_vec {
    ($other:expr, $self:expr) => {
        if $other.is_empty() {
            $self.clone()
        } else {
            $other.clone()
        }
    };
}

/// Macro to merge a collection field from an updated document into the current document.
///
/// This macro handles the merging logic for collection fields by:
/// - Replacing existing items with the same ID
/// - Adding new items that don't exist in the current document
///
/// # Arguments
/// * `$current` - The current document being updated
/// * `$updated` - The document containing updates
/// * `$new_enzmldoc` - The intermediate document for building the result
/// * `$field` - The field name to merge (e.g., vessels, reactions)
///
/// # Example
/// ```ignore
/// merge_field!(current, updated, new_enzmldoc, vessels);
/// ```
macro_rules! merge_field {
    ($current:expr, $updated:expr, $new_enzmldoc:expr, $field:ident) => {
        for item in $updated.$field.iter() {
            if let Some(index) = $current.$field.iter().position(|v| v.id == item.id) {
                // Merge the existing item at the found index using safe indexing
                if let Some(target_item) = $new_enzmldoc.$field.get_mut(index) {
                    *target_item = target_item.merge(item);
                } else {
                    // If index is somehow out of bounds, add as new item instead
                    $new_enzmldoc.$field.push(item.clone());
                }
            } else {
                // Add new item if it doesn't exist
                $new_enzmldoc.$field.push(item.clone());
            }
        }
    };
}

/// Trait for merging two objects of the same type
///
/// This trait defines the interface for merging objects, where the implementing
/// type can merge itself with another instance of the same type. The merge
/// operation typically prioritizes non-empty/non-null values from the `other`
/// object while preserving certain immutable fields from `self`.
pub trait Merge<T> {
    /// Merges this object with another object of the same type
    ///
    /// # Arguments
    /// * `other` - The object to merge into this one
    ///
    /// # Returns
    /// A new instance containing the merged data
    fn merge(&self, other: &T) -> Self;
}

/// Merges data from an updated EnzymeML document into the current document
///
/// This function performs intelligent merging of EnzymeML document fields:
/// - Vessels, small molecules, reactions, parameters, and complexes are merged
/// - Items with matching IDs are replaced
/// - New items are appended to the collections
///
/// # Arguments
/// * `current` - Mutable reference to the current document to be updated
/// * `updated` - Reference to the document containing updates
///
/// # Returns
/// - `Ok(())` if merge succeeds
/// - `Err(String)` if merge fails for any reason
///
/// # Implementation Details
/// Uses an intermediate document to build the merged result, then replaces
/// the current document to ensure atomic updates. All operations use safe
/// indexing to prevent panics.
pub fn merge_enzymeml_documents(
    current: &mut EnzymeMLDocument,
    updated: &EnzymeMLDocument,
) -> Result<(), String> {
    // Clone the current document to create an intermediate for merging
    // Clone operations on standard Rust types should not panic
    let mut intermediate = current.clone();

    // Merge all collection fields from the updated document
    // All operations use safe indexing (get_mut) to prevent panics
    merge_field!(current, updated, intermediate, vessels);
    merge_field!(current, updated, intermediate, small_molecules);
    merge_field!(current, updated, intermediate, reactions);
    merge_field!(current, updated, intermediate, parameters);
    merge_field!(current, updated, intermediate, complexes);
    merge_field!(current, updated, intermediate, proteins);

    // Atomically replace the current document with the merged result
    *current = intermediate;
    Ok(())
}

impl Merge<Vessel> for Vessel {
    /// Merges this vessel with another vessel
    ///
    /// The merge prioritizes non-empty values from the `other` vessel while
    /// preserving the original vessel's ID (which is never overwritable).
    ///
    /// # Arguments
    /// * `other` - The vessel to merge into this one
    ///
    /// # Returns
    /// A new `Vessel` instance with merged data
    fn merge(&self, other: &Vessel) -> Vessel {
        let name = merge_string!(other.name, self.name);

        Vessel {
            id: self.id.clone(), // Special case: never overwritable
            name,
            volume: other.volume,
            unit: other.unit.clone(),
            constant: other.constant,
        }
    }
}

impl Merge<SmallMolecule> for SmallMolecule {
    /// Merges this small molecule with another small molecule
    ///
    /// The merge prioritizes non-empty/non-null values from the `other` small molecule
    /// while preserving the original molecule's ID (which is never overwritable).
    /// Chemical identifiers, references, and other metadata are merged intelligently.
    ///
    /// # Arguments
    /// * `other` - The small molecule to merge into this one
    ///
    /// # Returns
    /// A new `SmallMolecule` instance with merged data
    fn merge(&self, other: &SmallMolecule) -> SmallMolecule {
        let name = merge_string!(other.name, self.name);
        let vessel_id = merge_option!(other.vessel_id, self.vessel_id);
        let canonical_smiles = merge_option!(other.canonical_smiles, self.canonical_smiles);
        let inchi = merge_option!(other.inchi, self.inchi);
        let inchikey = merge_option!(other.inchikey, self.inchikey);
        let synonymous_names = merge_vec!(other.synonymous_names, self.synonymous_names);
        let references = merge_vec!(other.references, self.references);

        SmallMolecule {
            id: self.id.clone(), // Special case: never overwritable
            name,
            constant: other.constant,
            vessel_id,
            canonical_smiles,
            inchi,
            inchikey,
            synonymous_names,
            references,
        }
    }
}

impl Merge<Protein> for Protein {
    /// Merges this protein with another protein
    ///
    /// The merge prioritizes non-empty/non-null values from the `other` protein
    /// while preserving the original protein's ID and sequence (which are never
    /// overwritable to prevent data corruption and hallucinations).
    ///
    /// # Arguments
    /// * `other` - The protein to merge into this one
    ///
    /// # Returns
    /// A new `Protein` instance with merged data
    fn merge(&self, other: &Protein) -> Protein {
        let name = merge_string!(other.name, self.name);
        let vessel_id = merge_option!(other.vessel_id, self.vessel_id);
        let ecnumber = merge_option!(other.ecnumber, self.ecnumber);
        let organism = merge_option!(other.organism, self.organism);
        let organism_tax_id = merge_option!(other.organism_tax_id, self.organism_tax_id);
        let references = merge_vec!(other.references, self.references);

        Protein {
            id: self.id.clone(), // Special case: never overwritable
            name,
            constant: other.constant,
            sequence: self.sequence.clone(), // Special case: never overwritable (prevent hallucinations)
            vessel_id,
            ecnumber,
            organism,
            organism_tax_id,
            references,
        }
    }
}

impl Merge<Reaction> for Reaction {
    /// Merges this reaction with another reaction
    ///
    /// The merge prioritizes non-empty/non-null values from the `other` reaction
    /// while preserving the original reaction's ID (which is never overwritable).
    /// Reaction participants (reactants, products, modifiers) and kinetic laws
    /// are merged intelligently.
    ///
    /// # Arguments
    /// * `other` - The reaction to merge into this one
    ///
    /// # Returns
    /// A new `Reaction` instance with merged data
    fn merge(&self, other: &Reaction) -> Reaction {
        let name = merge_string!(other.name, self.name);
        let kinetic_law = merge_option!(other.kinetic_law, self.kinetic_law);
        let reactants = merge_vec!(other.reactants, self.reactants);
        let products = merge_vec!(other.products, self.products);
        let modifiers = merge_vec!(other.modifiers, self.modifiers);

        Reaction {
            id: self.id.clone(), // Special case: never overwritable
            name,
            reversible: other.reversible,
            kinetic_law,
            reactants,
            products,
            modifiers,
        }
    }
}

impl Merge<Parameter> for Parameter {
    /// Merges this parameter with another parameter
    ///
    /// The merge prioritizes non-empty/non-null values from the `other` parameter
    /// while preserving the original parameter's ID (which is never overwritable).
    /// All parameter properties including bounds, fitting information, and statistical
    /// data are merged intelligently.
    ///
    /// # Arguments
    /// * `other` - The parameter to merge into this one
    ///
    /// # Returns
    /// A new `Parameter` instance with merged data
    fn merge(&self, other: &Parameter) -> Parameter {
        let name = merge_string!(other.name, self.name);
        let symbol = merge_string!(other.symbol, self.symbol);
        let value = merge_option_direct!(other.value, self.value);
        let unit = merge_option!(other.unit, self.unit);
        let initial_value = merge_option_direct!(other.initial_value, self.initial_value);
        let upper_bound = merge_option_direct!(other.upper_bound, self.upper_bound);
        let lower_bound = merge_option_direct!(other.lower_bound, self.lower_bound);
        let fit = merge_option_direct!(other.fit, self.fit);
        let stderr = merge_option_direct!(other.stderr, self.stderr);

        Parameter {
            id: self.id.clone(), // Special case: never overwritable
            name,
            symbol,
            value,
            unit,
            initial_value,
            upper_bound,
            lower_bound,
            fit,
            stderr,
            constant: other.constant,
        }
    }
}

impl Merge<Complex> for Complex {
    /// Merges this complex with another complex
    ///
    /// The merge prioritizes non-empty/non-null values from the `other` complex
    /// while preserving the original complex's ID (which is never overwritable).
    /// Complex participants and vessel associations are merged intelligently.
    ///
    /// # Arguments
    /// * `other` - The complex to merge into this one
    ///
    /// # Returns
    /// A new `Complex` instance with merged data
    fn merge(&self, other: &Complex) -> Complex {
        let name = merge_string!(other.name, self.name);
        let vessel_id = merge_option!(other.vessel_id, self.vessel_id);
        let participants = merge_vec!(other.participants, self.participants);

        Complex {
            id: self.id.clone(), // Special case: never overwritable
            name,
            constant: other.constant,
            vessel_id,
            participants,
        }
    }
}
