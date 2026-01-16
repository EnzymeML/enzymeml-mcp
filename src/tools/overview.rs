use std::collections::HashSet;

use enzymeml::prelude::{
    Complex, EnzymeMLDocument, Equation, EquationType, Measurement, MeasurementData,
    ModifierElement, Parameter, Protein, Reaction, ReactionElement, SmallMolecule, Vessel,
};
use meval::tokenizer::Token;
use serde::Serialize;

/// Macro to generate `From<&Type> for EntityMap` implementations for types with `id` and `name` fields
macro_rules! impl_entity_map_with_name {
    ($($type:ty),*) => {
        $(
            impl From<&$type> for EntityMap {
                fn from(item: &$type) -> Self {
                    Self {
                        id: item.id.clone(),
                        name: Some(item.name.clone()),
                    }
                }
            }
        )*
    };
}

/// Macro to generate `From<&Type> for EntityMap` implementations for types with only an ID field
macro_rules! impl_entity_map_without_name {
    ($($type:ty => $id_field:ident),*) => {
        $(
            impl From<&$type> for EntityMap {
                fn from(item: &$type) -> Self {
                    Self {
                        id: item.$id_field.clone(),
                        name: None,
                    }
                }
            }
        )*
    };
}

/// Macro to generate struct initialization with field mappings
/// Usage: build_overview!(source_var, target_field => source_field => TargetType, ...)
/// If source_field is omitted, it defaults to the same name as target_field
macro_rules! build_overview {
    // Case 1: target_field => source_field => Type (different names)
    ($source:ident, $( $target_field:ident => $source_field:ident => $target_type:ty ),* $(,)?) => {
        Self {
            $(
                $target_field: $source
                    .$source_field
                    .iter()
                    .map(|item| <$target_type>::from(item))
                    .collect(),
            )*
        }
    };
    // Case 2: field => Type (same name)
    ($source:ident, $( $field:ident => $target_type:ty ),* $(,)?) => {
        Self {
            $(
                $field: $source
                    .$field
                    .iter()
                    .map(|item| <$target_type>::from(item))
                    .collect(),
            )*
            ..Default::default()
        }
    };
}

/// A simplified representation of an entity with ID and optional name
///
/// This struct is used to create lightweight representations of various EnzymeML entities
/// for overview purposes. It implements Hash and Eq to allow use in HashSet collections.
#[derive(Eq, Hash, PartialEq, Clone, Debug, Serialize)]
pub struct EntityMap {
    /// The unique identifier of the entity
    pub id: String,
    /// The optional name of the entity (skipped in serialization if None)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// A comprehensive overview of an EnzymeML document
///
/// This struct provides a high-level summary of all the major components
/// in an EnzymeML document, including vessels, molecules, reactions, measurements,
/// parameters, and equations.
#[derive(Default, Debug, Serialize)]
pub struct Overview {
    /// List of vessels in the document
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub vessels: Vec<EntityMap>,
    /// List of small molecules in the document
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub small_molecules: Vec<EntityMap>,
    /// List of proteins in the document
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub proteins: Vec<EntityMap>,
    /// List of complexes in the document
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub complexes: Vec<EntityMap>,
    /// List of reactions with detailed information
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub reactions: Vec<ReactionOverview>,
    /// List of measurements with time series data
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub measurements: Vec<MeasurementOverview>,
    /// List of parameters in the document
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<EntityMap>,
    /// List of equations with their parameters
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub equations: Vec<EquationOverview>,
}

impl From<&EnzymeMLDocument> for Overview {
    /// Creates an overview from an EnzymeML document
    ///
    /// This implementation extracts all major components from the document
    /// and creates simplified representations for overview purposes.
    /// It also enriches reactions with their kinetic law information.
    fn from(enzmldoc: &EnzymeMLDocument) -> Self {
        let mut overview = build_overview!(
            enzmldoc,
            vessels => EntityMap,
            small_molecules => EntityMap,
            proteins => EntityMap,
            complexes => EntityMap,
            parameters => EntityMap,
            reactions => ReactionOverview,
            measurements => MeasurementOverview
        );

        // Handle equations
        overview.equations = enzmldoc
            .equations
            .iter()
            .map(|equation| EquationOverview::from_equation(equation, enzmldoc))
            .collect();

        for reaction in overview.reactions.iter_mut() {
            if let Some(doc_reaction) = enzmldoc
                .reactions
                .iter()
                .find(|r| r.id == reaction.id.as_ref().unwrap().id)
            {
                reaction.set_kinetic_law(doc_reaction, enzmldoc);
            }
        }

        overview
    }
}

impl_entity_map_with_name!(Vessel, SmallMolecule, Protein, Complex, Reaction);
impl_entity_map_without_name!(
    ReactionElement => species_id,
    ModifierElement => species_id,
    MeasurementData => species_id,
    Parameter => symbol
);

/// Overview of a reaction including its participants and kinetic law
///
/// This struct provides a comprehensive view of a reaction, including
/// its reactants, products, modifiers, and associated kinetic law equation.
#[derive(Default, Debug, Serialize)]
pub struct ReactionOverview {
    /// The reaction's ID and name information (flattened in serialization)
    #[serde(flatten)]
    pub id: Option<EntityMap>,
    /// List of reactant species
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub reactants: Vec<EntityMap>,
    /// List of product species
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub products: Vec<EntityMap>,
    /// List of modifier species (catalysts, inhibitors, etc.)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub modifiers: Vec<EntityMap>,
    /// The kinetic law equation if present
    pub kinetic_law: Option<EquationOverview>,
}

impl ReactionOverview {
    /// Sets the kinetic law for this reaction overview
    ///
    /// # Arguments
    /// * `reaction` - The source reaction containing the kinetic law
    /// * `enzmldoc` - The EnzymeML document for parameter resolution
    pub fn set_kinetic_law(&mut self, reaction: &Reaction, enzmldoc: &EnzymeMLDocument) {
        if let Some(kinetic_law) = &reaction.kinetic_law {
            self.kinetic_law = Some(EquationOverview::from_equation(kinetic_law, enzmldoc));
        }
    }
}

impl From<&Reaction> for ReactionOverview {
    /// Creates a reaction overview from a reaction
    ///
    /// This extracts the basic reaction information including reactants,
    /// products, and modifiers. The kinetic law is set separately.
    fn from(reaction: &Reaction) -> Self {
        let mut overview = build_overview!(
            reaction,
            reactants => EntityMap,
            products => EntityMap,
            modifiers => EntityMap,
        );

        overview.id = Some(EntityMap::from(reaction));

        overview
    }
}

/// Overview of a measurement containing time series data
///
/// This struct represents the species that are measured over time
/// in an experimental measurement.
#[derive(Debug, Serialize)]
pub struct MeasurementOverview {
    /// List of species for which time series data is available
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub time_series: Vec<InitialCondition>,
}

#[derive(Debug, Serialize)]
pub struct InitialCondition {
    /// The species id
    #[serde(flatten)]
    pub id: EntityMap,
    /// The initial condition value
    pub initial_condition: Option<f64>,
}

impl From<&Measurement> for MeasurementOverview {
    /// Creates a measurement overview from a measurement
    ///
    /// This extracts the species data to show which species
    /// have time series measurements available.
    fn from(measurement: &Measurement) -> Self {
        build_overview!(
            measurement,
            time_series => species_data => InitialCondition
        )
    }
}

impl From<&MeasurementData> for InitialCondition {
    fn from(measurement_data: &MeasurementData) -> Self {
        Self {
            id: EntityMap::from(measurement_data),
            initial_condition: measurement_data.initial,
        }
    }
}

/// Overview of an equation including its parameters and metadata
///
/// This struct represents an equation with information about
/// the parameters it uses and its type (ODE, rate law, etc.).
#[derive(Debug, Serialize)]
pub struct EquationOverview {
    /// List of parameters used in the equation
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<EntityMap>,
    /// The type of equation (ODE, RateLaw, etc.)
    pub equation_type: EquationType,
    /// The equation string
    pub equation: String,
}

impl EquationOverview {
    /// Creates an equation overview from an equation and document context
    ///
    /// This method parses the equation to extract parameter dependencies
    /// and creates a comprehensive overview including metadata.
    ///
    /// # Arguments
    /// * `equation` - The equation to create an overview for
    /// * `enzmldoc` - The EnzymeML document containing parameter definitions
    pub fn from_equation(equation: &Equation, enzmldoc: &EnzymeMLDocument) -> Self {
        let eq_string = equation.equation.clone();
        Self {
            parameters: Self::extract_parameters(&eq_string, &enzmldoc.parameters),
            equation_type: equation.equation_type.clone(),
            equation: eq_string,
        }
    }

    /// Extracts parameters used in an equation string
    ///
    /// This method parses the equation using meval and identifies
    /// which parameters from the document are referenced in the equation.
    ///
    /// # Arguments
    /// * `eq_string` - The equation string to parse
    /// * `parameters` - The list of available parameters from the document
    ///
    /// # Returns
    /// A vector of EntityMap representing the parameters used in the equation
    pub fn extract_parameters(eq_string: &str, parameters: &[Parameter]) -> Vec<EntityMap> {
        let expr: meval::Expr = eq_string.parse().unwrap();
        let parameters: Vec<EntityMap> = parameters.iter().map(EntityMap::from).collect();
        let mut included_params = HashSet::new();

        for symbol in expr.iter() {
            if let Token::Var(var) = symbol
                && let Some(parameter) = parameters.iter().find(|p| p.id == *var)
            {
                included_params.insert(parameter.clone());
            }
        }
        included_params.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use enzymeml::prelude::*;
    use toon_format::encode_default;

    use super::*;

    /// Test the overview generation functionality
    ///
    /// This test creates a sample EnzymeML document and generates
    /// an overview, then serializes it to TOON format for verification.
    #[test]
    fn test_overview() {
        let enzmldoc = create_enzmldoc();
        let overview = Overview::from(&enzmldoc);
        let toon = encode_default(&overview).unwrap();
        println!("{}", toon);
    }

    /// Creates a sample EnzymeML document for testing
    ///
    /// This function builds a comprehensive EnzymeML document with
    /// vessels, molecules, reactions, measurements, parameters, and equations
    /// to test the overview generation functionality.
    ///
    /// # Returns
    /// A fully constructed EnzymeML document for testing
    fn create_enzmldoc() -> EnzymeMLDocument {
        EnzymeMLDocumentBuilder::default()
            .name("EnzymeML Document 1")
            .to_vessels(
                VesselBuilder::default()
                    .id("V1")
                    .name("Vessel 1")
                    .volume(1.0)
                    .unit(UnitDefinition::default())
                    .build()
                    .unwrap(),
            )
            .to_small_molecules(
                SmallMoleculeBuilder::default()
                    .id("S1")
                    .name("Small Molecule 1")
                    .constant(true)
                    .build()
                    .unwrap(),
            )
            .to_small_molecules(
                SmallMoleculeBuilder::default()
                    .id("S2")
                    .name("Small Molecule 2")
                    .constant(true)
                    .build()
                    .unwrap(),
            )
            .to_proteins(
                ProteinBuilder::default()
                    .id("P1")
                    .name("Protein 1")
                    .constant(true)
                    .build()
                    .unwrap(),
            )
            .to_complexes(
                ComplexBuilder::default()
                    .id("C1")
                    .name("Complex 1")
                    .constant(true)
                    .build()
                    .unwrap(),
            )
            .to_reactions(
                ReactionBuilder::default()
                    .id("R1")
                    .name("Reaction 1")
                    .reversible(true)
                    .to_reactants(
                        ReactionElementBuilder::default()
                            .species_id("S1")
                            .build()
                            .unwrap(),
                    )
                    .to_products(
                        ReactionElementBuilder::default()
                            .species_id("S2")
                            .build()
                            .unwrap(),
                    )
                    .to_modifiers(
                        ModifierElementBuilder::default()
                            .species_id("P1")
                            .role(ModifierRole::Catalyst)
                            .build()
                            .unwrap(),
                    )
                    .kinetic_law(
                        EquationBuilder::default()
                            .species_id("v")
                            .equation_type(EquationType::RateLaw)
                            .equation("k1 * S1 * S2")
                            .build()
                            .unwrap(),
                    )
                    .build()
                    .unwrap(),
            )
            .to_measurements(
                MeasurementBuilder::default()
                    .id("M1")
                    .name("Measurement 1")
                    .to_species_data(
                        MeasurementDataBuilder::default()
                            .species_id("S1")
                            .build()
                            .unwrap(),
                    )
                    .to_species_data(
                        MeasurementDataBuilder::default()
                            .species_id("S2")
                            .build()
                            .unwrap(),
                    )
                    .build()
                    .unwrap(),
            )
            .to_parameters(
                ParameterBuilder::default()
                    .id("K1")
                    .symbol("k1")
                    .name("Parameter 1")
                    .build()
                    .unwrap(),
            )
            .to_parameters(
                ParameterBuilder::default()
                    .id("K2")
                    .symbol("k2")
                    .name("Parameter 2")
                    .build()
                    .unwrap(),
            )
            .to_parameters(
                ParameterBuilder::default()
                    .id("K3")
                    .symbol("k3")
                    .name("Parameter 3")
                    .build()
                    .unwrap(),
            )
            .to_equations(
                EquationBuilder::default()
                    .species_id("S1")
                    .equation_type(EquationType::Ode)
                    .equation("k2 * S1 - k3 * S2")
                    .build()
                    .unwrap(),
            )
            .build()
            .unwrap()
    }
}
