![DB Loada](docs/gfx/dbloada_logo.png)

# DBLOADA

*<sub>D-B load-uh</sub>*

## About

DBLOADA compiles data from various data sources and builds structured databases based on that data. It's main purpose
is to build databases optimized for AI Agents and AI Assistants to consume so that they can quickly get the exact
information they need by consuming minimal context. It's a commandline tool that reads a manifest file that contains
information about static and dynamic data sources and then use that information to build a structured database in a
SQL or Graph database. It also creates an Agent Skill that can be used by AI to understand how to query the database
efficiently. 

## Architecture
Written in Rust, DBLOADA uses a micro component architecture with dependency injection for modularity, extensibility, 
and testability. Each capability is defined as a trait with concrete implementations wired together through a central 
composition root (`ComponentAssembler`), making it straightforward to swap, extend, or mock any part of the system.

## Getting Started

```bash
cargo build
```

### Initialize a project

```bash
dbloada init                       # initialize current directory
dbloada init -d /path/to/dir       # initialize a specific directory
dbloada init -n my-project         # use an explicit project name
```

This creates a complete example project in the target directory:

- `dbloada.yaml` — project spec with 3 example tables (country, city, office) including columns and relationships
- `data/` — CSV data files for each table
- `scripts/` — empty directory for custom scripts

The project name defaults to the directory name, sanitized to a valid Kubernetes DNS label (RFC 1123).

### Load a project

```bash
dbloada load                        # load from current directory
dbloada load -d testdata/testproject # load from a specific directory
```

Reads the `dbloada.yaml` file from the given directory, parses the full project model (tables, columns, relationships, sources), and prints it to stdout.

## Running Tests

```bash
cargo test
```
