# DB Loada

DB Loada compiles data from various data sources and builds a structured database based on that data.

## About

Written in Rust, DB Loada uses a micro component architecture with dependency injection for modularity, extensibility, and testability. Each capability is defined as a trait with concrete implementations wired together through a central composition root (`ComponentAssembler`), making it straightforward to swap, extend, or mock any part of the system.

## Getting Started

```bash
cargo build
RUST_LOG=info cargo run
```

## Running Tests

```bash
cargo test
```
