# EnzymeML MCP Server

An MCP (Model Context Protocol) server that provides comprehensive tools for interacting with EnzymeML documents through the EnzymeML Suite desktop application. This server enables AI assistants and other MCP-compatible clients to read, modify, search, and visualize biochemical data stored in EnzymeML format.

## Overview

EnzymeML is a standardized data format for representing enzymatic reactions and experimental data in computational biology and biochemistry. This MCP server acts as a bridge between AI assistants and the EnzymeML Suite desktop application, allowing you to programmatically interact with EnzymeML documents, search external databases for biochemical entities, and generate visualizations of experimental measurements.

The server communicates with the EnzymeML Suite application running on your local machine and provides a rich set of tools for document manipulation, database searching, and data visualization. All data is exchanged using the TOON (Tree Object Oriented Notation) format, which provides a structured and efficient way to represent complex biochemical data.

## Features

### Document Reading and Exploration

- **Document Overview**: Generate high-level overviews of EnzymeML document structure and relationships, essential for understanding how different components are connected before performing edits
- **Full Document Reading**: Read complete EnzymeML document structures excluding measurements for optimal performance
- **Measurement Data Access**: Specifically fetch detailed experimental measurement data including time series, concentrations, and measured values

### Document Modification

- **Intelligent Document Extension**: Merge new data into existing EnzymeML documents with automatic validation and consistency checks. Supports both adding new items and performing surgical edits to existing items identified by ID
- **Selective Element Removal**: Remove specific elements from documents including complete objects (vessels, proteins, small molecules, reactions, parameters, complexes) or partial removal within reactions (reactants, products, modifiers)

### External Database Integration

- **UniProt Search**: Search the UniProt Knowledgebase for proteins with support for Boolean operators (AND, OR, NOT) and field-based filtering (accession, EC number, organism, protein name, sequence, etc.)
- **PubChem Search**: Search the PubChem database for small molecules with Boolean operators and field-based filtering (name, formula, InChI, InChIKey, SMILES, mass, etc.). Preferred over ChEBI for small molecule searches
- **ChEBI Search**: Search the ChEBI database for small molecules when PubChem results are insufficient

### Visualization

- **Measurement Plotting**: Generate SVG plots of measurements from EnzymeML documents. Can plot all measurements or a specified subset by measurement IDs, returning images in MultiImageResponse format

## Prerequisites

Before using this MCP server, ensure you have the following installed:

- **Rust**: The Rust programming language and Cargo package manager (version 1.70 or later recommended). Install from [rustup.rs](https://rustup.rs/)
- **EnzymeML Suite**: The EnzymeML Suite desktop application must be installed and running on your local machine. The server communicates with the Suite application to access EnzymeML documents
- **MCP Client**: An MCP-compatible client application (such as Cursor, Claude Desktop, or other MCP implementations) to connect to this server

## Building the Server

To build the EnzymeML MCP server from source, follow these steps:

1. **Clone the repository** (if you haven't already):

   ```bash
   git clone <repository-url>
   cd enzymeml-mcp
   ```

2. **Build the project** using Cargo:

   ```bash
   cargo build --release
   ```

   This will compile the server and create an executable at `target/release/enzymeml-mcp`.

3. **For development builds** (faster compilation, larger binary):

   ```bash
   cargo build
   ```

   Development builds are located at `target/debug/enzymeml-mcp`.

## Running the Server

The EnzymeML MCP server uses stdio (standard input/output) transport for communication with MCP clients. The server is designed to be launched by MCP-compatible clients and communicates via JSON-RPC messages over stdin/stdout.

### Manual Testing

To test the server manually, you can run it directly:

```bash
cargo run --release
```

However, note that the server expects JSON-RPC messages on stdin and will not produce useful output without a proper MCP client connection.

#### Example Configuration (Cursor)

For Cursor, you might configure the server in your MCP settings file:

```json
{
  "mcpServers": {
    "enzymeml": {
      "command": "/path/to/enzymeml-mcp/target/release/enzymeml-mcp",
      "args": []
    }
  }
}
```

#### Example Configuration (Claude Desktop)

For Claude Desktop, add to your `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "enzymeml": {
      "command": "/path/to/enzymeml-mcp/target/release/enzymeml-mcp",
      "args": []
    }
  }
}
```

**Important**: Replace `/path/to/enzymeml-mcp` with the actual path to your compiled binary.

## Available Tools

The server exposes the following tools for interacting with EnzymeML documents:

### `list_documents`

Lists all EnzymeML documents available from the Suite application, returned in TOON format. Useful for discovering which documents are available when working with multiple documents.

### `enzymeml_document_overview`

Generates a high-level overview of the EnzymeML document structure and relationships in TOON format. Essential for understanding document structure before performing edits or when you need to understand how components are connected.

### `read_enzymeml_document`

Reads the complete EnzymeML document structure from the Suite desktop application, excluding measurements for performance. Returns data in TOON format. Use `read_measurements` if you specifically need measurement data.

### `read_measurements`

Specifically fetches measurement data from the EnzymeML document, including time series, concentrations, and measured values. Returns detailed experimental data in TOON format.

### `extend_enzymeml_document`

Intelligently merges new data into the existing EnzymeML document. Supports both adding new items and performing surgical edits to existing items (identified by ID). Performs automatic validation and consistency checks. Always ask for confirmation before submitting changes.

### `remove_from_enzymeml_document`

Selectively removes elements from the EnzymeML document. Supports complete removal of objects (vessels, proteins, small molecules, reactions, parameters, complexes) and partial removal within reactions (reactants, products, modifiers). Requires knowledge of existing IDs from `enzymeml_document_overview`.

### `search_uniprot`

Searches the UniProt Knowledgebase for proteins. Supports Boolean operators (AND, OR, NOT) and field-based filtering. Filterable fields include accession, EC number, organism name, protein name, and sequence.

### `search_pubchem`

Searches the PubChem database for small molecules. Supports Boolean operators and field-based filtering (name, formula, InChI, InChIKey, SMILES, mass, etc.). Preferred over ChEBI for small molecule searches.

### `search_chebi`

Searches the ChEBI database for small molecules. Supports Boolean operators and field-based filtering. Use when PubChem results are insufficient.

### `plot_measurements`

Generates SVG plots of measurements from the EnzymeML document. Can plot all measurements or a specified subset by measurement IDs. Returns a list of images in MultiImageResponse format.

## Usage Workflow

When working with EnzymeML documents through this MCP server, follow this recommended workflow:

1. **Start with instructions**: Call `how_to_use_the_enzymeml_suite` to understand the available tools and their usage patterns
2. **List available documents**: Use `list_documents` to see what documents are available (if working with multiple documents)
3. **Get document overview**: Always call `enzymeml_document_overview` before performing any modifications to understand the document structure and discover existing IDs
4. **Search external databases**: Before adding new proteins or small molecules, search UniProt, PubChem, or ChEBI to enrich your data with standardized metadata
5. **Read document data**: Use `read_enzymeml_document` or `read_measurements` as needed to access specific data
6. **Modify documents**: Use `extend_enzymeml_document` for additions and edits, or `remove_from_enzymeml_document` for removals. Always verify IDs exist before modifications
7. **Visualize data**: Use `plot_measurements` to generate visualizations of experimental measurements

## Dependencies

This project relies on the following key dependencies:

- **rmcp**: Model Context Protocol implementation for Rust
- **enzymeml**: Rust library for working with EnzymeML documents (from the EnzymeML organization)
- **tokio**: Async runtime for Rust
- **toon-format**: TOON format encoding and decoding
- **reqwest**: HTTP client for external database API calls
- **quill**: Plotting library for generating SVG visualizations

See `Cargo.toml` for the complete list of dependencies and their versions.
