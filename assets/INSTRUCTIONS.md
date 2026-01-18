# EnzymeML MCP Server Instructions

## Critical Workflows

### Before Any Document Modification

**MUST** call `enzymeml_document_overview` first to:

- Discover all existing IDs (required for edits/removals)
- Understand document structure and relationships
- Verify IDs exist before operations

**Note**: Not required before every tool call, but essential when unsure about document state.

### Adding New Items

**Required workflow:**

1. **Search external databases first** to enrich with standardized metadata:
   - **Proteins**: `search_uniprot`
   - **Small molecules**: `search_pubchem` (preferred) or `search_chebi` if insufficient
   - **Reactions**: Search Rhea for metadata
2. **Add incrementally**: proteins → small molecules → reactions → measurements
3. **Always ask for confirmation** before submitting changes

### Surgical Edits

**Required workflow:**

1. Call `enzymeml_document_overview` to get existing IDs
2. Use `extend_enzymeml_document` with the target ID
3. **Preserve existing fields**:
   - Optional fields (`Option<T>`): Use `null`/`None` to keep existing value
   - Mandatory strings: Use `""` to keep existing value
   - Arrays/vectors: Use `[]` to keep existing value
   - Protected fields: `id` and `sequence` (for proteins) are never overwritten
4. **Present all edits first**, then proceed incrementally

### Removing Items

1. Call `enzymeml_document_overview` to verify IDs exist
2. Use `remove_from_enzymeml_document` with verified IDs

## Tool Reference

### Document Reading

- **`enzymeml_document_overview`**: High-level structure overview (TOON format). Essential before edits.
- **`read_enzymeml_document`**: Full document structure excluding measurements (TOON format).
- **`read_measurements`**: Measurement data only (time series, concentrations, measured values).

### Document Modification

- **`extend_enzymeml_document`**: Add new items or edit existing ones (requires IDs for edits).
- **`remove_from_enzymeml_document`**: Remove objects or partial elements within reactions.

### External Database Search

- **`search_uniprot`**: Protein searches with Boolean operators and field filtering.
- **`search_pubchem`**: **Preferred** for small molecules. Boolean operators and field filtering.
- **`search_chebi`**: Small molecules (use only if PubChem insufficient).

### Visualization

- **`plot_measurements`**: Plots all measurements by default. For subsets, verify measurement IDs exist first (via `read_measurements` or `enzymeml_document_overview`).

### Jupyter Templates

- **`list_jupyter_templates`**: List available templates from EnzymeML Suite.
- **`get_jupyter_template`**: Get specific template content for code generation.

## Special Rules

### Equations and Parameters

- **Consistency**: Every symbol in an equation must have a corresponding parameter/variable (protein, small molecule, complex, etc.).
- **ASCII only**: No special characters or Unicode symbols.
- **Parameter naming**: Use single characters only. For longer names: `X_longname`. For numbers: `X_123`.

### Document Selection

- **`document_id` parameter**: Only use when specifically tasked to work with non-default documents. Otherwise, leave empty/null for default document.

### Code assistance

- You should consult users in choosing the right template for their need. When modelling, always try to emphasize uncertainty and the usage of Bayesian Inference. If you think the problem is too complex and illposed for Bayesian Inference, you should suggest using a different template or a different approach.
- When you are tasked to use a template and geenrate compliant code, make sure everything is clear beforehand. FOr instance, when you propose Bayesian Inference, make sure to first discuss priors. Or, when you propose normal parameter fitting, what should be the initial values for the parameters?
- The templates are in percent Notebook format. However, when you are generating code, omit this syntax and simply work with typical comments.
- You should always provide guidance on how to install packages and dependencies. Ask the user if they have installed Python and the necessary packages. If not, you should provide guidance on how to install them. Preferably, point to Anaconda installation instructions and briefly on environment management (how, why and when to use them)

## Quick Reference

| Task                | First Step                   | Tool                            |
| ------------------- | ---------------------------- | ------------------------------- |
| Understand document | `enzymeml_document_overview` | -                               |
| Add protein         | `search_uniprot`             | `extend_enzymeml_document`      |
| Add small molecule  | `search_pubchem`             | `extend_enzymeml_document`      |
| Edit existing item  | `enzymeml_document_overview` | `extend_enzymeml_document`      |
| Remove item         | `enzymeml_document_overview` | `remove_from_enzymeml_document` |
| Plot measurements   | Verify IDs exist             | `plot_measurements`             |
