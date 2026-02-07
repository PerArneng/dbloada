# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Run

```bash
cargo build          # compile
cargo run            # run (add RUST_LOG=info for log output)
cargo test           # run all tests
cargo test <name>    # run a single test by name
```

Requires Rust edition 2024.

## Architecture

dbloada uses a **trait + implementation** pattern with a **composition root** for dependency injection.

- **`src/traits/`** — Public trait definitions (one file per trait, re-exported from `mod.rs`). These are the abstractions the rest of the codebase depends on.
- **`src/components/`** — Concrete implementations. Each component lives in its own subdirectory (e.g. `components/logger/`), with the struct named `<Name>Impl` in `<name>_impl.rs`.
- **`src/component_assembler.rs`** — The composition root (`ComponentAssembler`). It wires concrete implementations to their traits and returns `Box<dyn Trait>`. Each factory method is named after the trait in snake_case (e.g. `Logger` → `logger()`, `DbLoadaEngine` → `db_loada_engine()`).
- **`src/main.rs`** — Entry point. Only interacts with `ComponentAssembler` and traits, never with concrete implementations.

Dependencies are injected as `Box<dyn Trait>` via constructor parameters (`new()`).

## Skills

- **`dbloada-component`** — Use when creating a new component. Covers the full workflow: trait, implementation, and `ComponentAssembler` wiring.
