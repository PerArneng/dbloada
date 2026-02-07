# DB Loada

DB Loada compiles data from various data sources and builds a structured database based on that data.

## About

Written in Rust, DB Loada uses a micro component architecture with dependency injection for modularity, extensibility, and testability. Each capability is defined as a trait with concrete implementations wired together through a central composition root (`ComponentAssembler`), making it straightforward to swap, extend, or mock any part of the system.

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

This creates a `dbloada.yaml` file in the target directory. The project name defaults to the directory name, sanitized to a valid Kubernetes DNS label (RFC 1123).

## Running Tests

```bash
cargo test
```
