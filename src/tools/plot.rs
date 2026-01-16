//! Tools for plotting EnzymeML measurement data
//!
//! This module provides functionality for visualizing measurement data from EnzymeML documents,
//! including plotting time series data for different species with color-coded series.

use base64::{Engine, engine::general_purpose};
use enzymeml::prelude::{Complex, EnzymeMLDocument, MeasurementData, Protein, SmallMolecule};
use quill::{
    color::Color,
    plot::Plot,
    prelude::{Grid, Legend, Marker},
    series::Series,
};
use std::collections::HashMap;

/// Color palette for plotting measurement data series
///
/// This array provides a consistent set of colors for visualizing different
/// species in measurement plots. Colors are cycled through when there are
/// more series than available colors.
const PLOT_COLORS: [Color; 10] = [
    Color::Blue,
    Color::Red,
    Color::Green,
    Color::Orange,
    Color::Purple,
    Color::Cyan,
    Color::Magenta,
    Color::Yellow,
    Color::Pink,
    Color::Brown,
];

/// Macro to build plot and return base64-encoded PNG string.
///
/// Quill requires compile-time known array sizes, so we match on length.
/// This macro generates a plot with the specified measurement data and converts
/// it to a PNG image encoded as base64 for display.
///
/// # Arguments
/// * `$measurement` - The measurement object containing metadata like name
/// * `$data` - Vector of series data to plot
/// * `$($idx:literal),+` - Compile-time indices for accessing data array elements
///
/// # Returns
/// A base64-encoded PNG string
macro_rules! build_plot_image {
    ($measurement:expr, $data:expr, $($idx:literal),+) => {{
        let title = format!("{} - {}", $measurement.name, $measurement.id);
        let plot = Plot::builder()
            .dimensions((400, 250))
            .title(&title)
            .x_label("Time")
            .y_label("Concentration")
            .legend(Legend::TopRightOutside)
            .grid(Grid::Solid)
            .data([$($data[$idx].clone()),+])
            .build();
        let plot_bytes = plot.to_png_bytes(3.0)
            .map_err(|e| anyhow::anyhow!("Failed to generate plot PNG: {}", e))?;
        let plot_base64 = general_purpose::STANDARD.encode(&plot_bytes);
        plot_base64
    }};
}

/// Plots a specific measurement from an EnzymeML document
///
/// This function creates a visualization of measurement data for a given measurement ID.
/// It processes all species data within the measurement and generates a plot with
/// different colored series for each species.
///
/// # Arguments
/// * `measurement_id` - The unique identifier of the measurement to plot
/// * `enzmldoc` - Reference to the EnzymeML document containing the measurement
///
/// # Returns
/// * `Ok(String)` - Base64-encoded PNG image bytes on success
/// * `Err(anyhow::Error)` - Error if measurement not found or plotting fails
///
/// # Errors
/// * Returns error if measurement ID is not found in the document
/// * Returns error if no valid series data is available for plotting
/// * Returns error if PNG generation fails
pub fn plot_measurement(
    measurement_id: &str,
    enzmldoc: &EnzymeMLDocument,
) -> anyhow::Result<String> {
    let measurement = enzmldoc
        .measurements
        .iter()
        .find(|m| m.id == measurement_id)
        .ok_or_else(|| anyhow::anyhow!("Measurement {} not found", measurement_id))?;

    // Build species lookup map for O(1) access
    let all_species = collect_species(enzmldoc);
    let species_map: HashMap<&str, &Species> = all_species.iter().map(|s| (s.id(), s)).collect();

    // Sort and process measurement data by species_id
    let mut sorted_species_data: Vec<_> = measurement.species_data.iter().collect();
    sorted_species_data.sort_by_key(|meas_data| &meas_data.species_id);

    let data: Vec<_> = sorted_species_data
        .iter()
        .enumerate()
        .filter_map(|(idx, meas_data)| {
            species_map
                .get(meas_data.species_id.as_str())
                .and_then(|species| {
                    measurement_to_series(
                        meas_data,
                        species,
                        PLOT_COLORS[idx % PLOT_COLORS.len()].clone(),
                    )
                    .ok()
                })
        })
        .take(PLOT_COLORS.len())
        .collect();

    if data.is_empty() {
        return Err(anyhow::anyhow!("No valid series data found"));
    }

    let base64_string = match data.len() {
        1 => build_plot_image!(measurement, data, 0),
        2 => build_plot_image!(measurement, data, 0, 1),
        3 => build_plot_image!(measurement, data, 0, 1, 2),
        4 => build_plot_image!(measurement, data, 0, 1, 2, 3),
        5 => build_plot_image!(measurement, data, 0, 1, 2, 3, 4),
        6 => build_plot_image!(measurement, data, 0, 1, 2, 3, 4, 5),
        7 => build_plot_image!(measurement, data, 0, 1, 2, 3, 4, 5, 6),
        8 => build_plot_image!(measurement, data, 0, 1, 2, 3, 4, 5, 6, 7),
        9 => build_plot_image!(measurement, data, 0, 1, 2, 3, 4, 5, 6, 7, 8),
        _ => build_plot_image!(measurement, data, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9),
    };

    Ok(base64_string)
}

/// Converts measurement data and species information into a plot series
///
/// This function takes raw measurement data (time and concentration values) and
/// combines it with species metadata to create a formatted series suitable for plotting.
///
/// # Arguments
/// * `meas_data` - The measurement data containing time points and values
/// * `species` - The species information (name, ID, etc.)
/// * `color` - The color to use for this series in the plot
///
/// # Returns
/// * `Ok(Series)` - A configured series ready for plotting
/// * `Err(anyhow::Error)` - Error if measurement data is empty or invalid
///
/// # Errors
/// Returns error if the measurement data contains no time points
fn measurement_to_series<'a>(
    meas_data: &'a MeasurementData,
    species: &'a Species,
    color: Color,
) -> anyhow::Result<Series<'a, f64>> {
    if meas_data.time.is_empty() {
        return Err(anyhow::anyhow!("Measurement data is empty"));
    }

    let data: Vec<_> = meas_data
        .time
        .iter()
        .zip(meas_data.data.iter())
        .map(|(t, d)| (*t, *d))
        .collect();

    Ok(Series::builder()
        .name(species.name())
        .color(color)
        .data(data)
        .marker(Marker::Circle)
        .build())
}

/// Unified species type for handling different kinds of biochemical entities
///
/// This enum provides a common interface for working with different types of
/// species in EnzymeML documents, including proteins, small molecules, and complexes.
/// It allows for uniform handling of species data regardless of the underlying type.
enum Species {
    /// A protein species
    Protein(Protein),
    /// A small molecule species
    SmallMolecule(SmallMolecule),
    /// A complex species (combination of other species)
    Complex(Complex),
}

impl Species {
    /// Gets the display name of the species
    ///
    /// # Returns
    /// The human-readable name of the species
    fn name(&self) -> &str {
        match self {
            Species::Protein(protein) => &protein.name,
            Species::SmallMolecule(small_molecule) => &small_molecule.name,
            Species::Complex(complex) => &complex.name,
        }
    }

    /// Gets the unique identifier of the species
    ///
    /// # Returns
    /// The unique ID string for this species
    fn id(&self) -> &str {
        match self {
            Species::Protein(protein) => &protein.id,
            Species::SmallMolecule(small_molecule) => &small_molecule.id,
            Species::Complex(complex) => &complex.id,
        }
    }
}

impl From<&SmallMolecule> for Species {
    /// Converts a SmallMolecule reference into a Species enum variant
    fn from(small_molecule: &SmallMolecule) -> Self {
        Species::SmallMolecule(small_molecule.clone())
    }
}

impl From<&Protein> for Species {
    /// Converts a Protein reference into a Species enum variant
    fn from(protein: &Protein) -> Self {
        Species::Protein(protein.clone())
    }
}

impl From<&Complex> for Species {
    /// Converts a Complex reference into a Species enum variant
    fn from(complex: &Complex) -> Self {
        Species::Complex(complex.clone())
    }
}

/// Collects all species from an EnzymeML document into a unified vector
///
/// This function gathers all different types of species (proteins, small molecules,
/// and complexes) from an EnzymeML document and returns them as a single vector
/// of unified Species enum variants.
///
/// # Arguments
/// * `enzmldoc` - Reference to the EnzymeML document to collect species from
///
/// # Returns
/// A vector containing all species from the document as Species enum variants
fn collect_species(enzmldoc: &EnzymeMLDocument) -> Vec<Species> {
    let mut species = Vec::new();
    for protein in &enzmldoc.proteins {
        species.push(Species::from(protein));
    }
    for small_molecule in &enzmldoc.small_molecules {
        species.push(Species::from(small_molecule));
    }
    for complex in &enzmldoc.complexes {
        species.push(Species::from(complex));
    }
    species
}
