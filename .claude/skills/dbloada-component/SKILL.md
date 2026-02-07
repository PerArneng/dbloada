---
name: dbloada-component
description: instructions on how to create a new component for db loada. Use this skill when the user request "new component for dbloada" or "create a new component".
---

# How to Create a New Component

This guide uses a component called `Foo` as an example. Replace `Foo`/`foo` with the actual component name.

## Step 1: Create the trait in `src/traits/`

Create `src/traits/foo.rs` with the trait definition:

```rust
pub trait Foo {
    // define the trait methods here
}
```

Register it in `src/traits/mod.rs` by adding:

```rust
pub mod foo;
pub use foo::Foo;
```

## Step 2: Create the component implementation in `src/components/`

Create the directory `src/components/foo/` with two files.

`src/components/foo/foo_impl.rs` — the concrete implementation:

```rust
use crate::traits::Foo;

pub struct FooImpl {
    // dependencies as fields, e.g.:
    // logger: Box<dyn Logger>,
}

impl FooImpl {
    pub fn new(/* dependencies */) -> Self {
        FooImpl { /* fields */ }
    }
}

impl Foo for FooImpl {
    // implement trait methods here
}
```

`src/components/foo/mod.rs` — module declaration and re-export:

```rust
pub mod foo_impl;

pub use foo_impl::FooImpl;
```

Register the component module in `src/components/mod.rs` by adding:

```rust
pub mod foo;
```

## Step 3: Add a factory function to `ComponentAssembler`

In `src/component_assembler.rs`, add a function that returns `Box<dyn Foo>`. The function name should match the trait name in snake_case. For example, the `Logger` trait gets a `logger()` function, `DbLoadaEngine` gets `db_loada_engine()`, and `Foo` gets `foo()`.

```rust
pub fn foo(&self) -> Box<dyn Foo> {
    // wire up dependencies, e.g.:
    // let logger: Box<dyn Logger> = Box::new(EnvLogger::new());
    Box::new(FooImpl::new(/* pass dependencies */))
}
```

Add the necessary `use` imports at the top of the file:

```rust
use crate::components::foo::FooImpl;
use crate::traits::Foo;
```

## Conventions

- **Traits** live in `src/traits/` — one file per trait, re-exported from `src/traits/mod.rs`.
- **Component implementations** live in `src/components/<name>/` — the struct is named `<Name>Impl` (e.g. `FooImpl`) and lives in `<name>_impl.rs`, re-exported from the module's `mod.rs`.
- **Dependencies** are injected as `Box<dyn Trait>` fields via the constructor `new()`.
- **ComponentAssembler** (`src/component_assembler.rs`) is the composition root. It wires concrete implementations to traits and returns `Box<dyn Trait>`. Each factory function is named after the trait in snake_case.
- **`main.rs`** only interacts with `ComponentAssembler` and traits — never with concrete implementations directly.
