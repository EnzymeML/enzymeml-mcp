# EnzymeML MCP Server Instructions

## Tool Usage Rules

### Prerequisites and Required Tools

**Before performing ANY document modifications:**

- **MUST** call `enzymeml_document_overview` first to discover all existing IDs and understand document structure
- This is critical for surgical edits and removals - without correct IDs, operations will fail
- This should also be called before any tools that require knowledge of the document structure. It is not mandatory to call this tool before every tool call, but if unsure, call it first.

**Before adding new items:**

- **MUST** search external databases first to enrich data with standardized metadata:
  - Proteins: Use `search_uniprot` before adding
  - Small molecules: Use `search_pubchem` (preferred) or `search_chebi` if PubChem is insufficient
  - Reactions: Search Rhea for metadata (before adding)

**Before plotting specific measurements:**

- **MUST** first read measurement IDs (via `read_measurements` or `enzymeml_document_overview`) to verify they exist

### When to Use Which Tool

#### Document Reading

- **`enzymeml_document_overview`**: Use first to understand document structure and relationships. Essential before any edits.
- **`read_enzymeml_document`**: Use when you need the full document structure (excluding measurements for performance).
- **`read_measurements`**: Use when you specifically need measurement data (time series, concentrations, measured values).

#### Document Modification

- **`extend_enzymeml_document`**:
  - **Adding new items**: Add incrementally (proteins → small molecules → reactions → measurements). Always search external databases first.
  - **Surgical edits**: Requires existing IDs from `enzymeml_document_overview`. Use null/empty values to preserve existing fields.
  - **Always**: Ask for confirmation before submitting. For multiple edits, present all edits first, then proceed incrementally.

- **`remove_from_enzymeml_document`**: Requires existing IDs from `enzymeml_document_overview`. Verify IDs exist before removal.

#### External Database Search

- **`search_uniprot`**: Use for protein searches before adding proteins to document.
- **`search_pubchem`**: **Preferred** for small molecule searches. Use before adding small molecules.
- **`search_chebi`**: Use only when PubChem results are insufficient.

#### Visualization

- **`plot_measurements`**: Plots all measurements by default. For subsets, verify measurement IDs exist first.

#### Adding equations and parameters

- There should be no inconsistencies within equations. For every symbol in the equation, there should be a corresponding parameter or variable (protein, small molecule, complex, etc.).
- Equations have to be ascii only. No special characters or unicode symbols.
- **IMPORTANT**: When adding parameters, use single characters only. If you want to use a longer name, use a single letter followed by an underscore and the longer name. If you want to add a parameter with a number in the name, use a single letter followed by an underscore and the number.

