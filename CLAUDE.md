# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`assert-struct` is a procedural macro library for ergonomic structural assertions in Rust tests. It enables deep, partial matching of complex data structures without manually referencing every field - particularly useful for testing typed JSON responses and other nested data structures.

### Core Features (Planned)
- **Partial matching**: Check only the fields you care about
- **Nested struct support**: Deep assertions without verbose field access chains
- **Flexible matchers**: Comparison operators (>=, <=, etc.), regex patterns, custom functions
- **Collection support**: Assert on Vec, HashMap, and other collections
- **Custom error messages**: Clear failure output showing exactly what didn't match
- **Pattern matching syntax**: Intuitive DSL for complex assertions

### Example Use Case
```rust
// Instead of:
assert_eq!(response.user.profile.settings.notifications.email, true);
assert!(response.user.profile.age >= 18);
assert!(response.items.len() > 0);

// You can write:
assert_struct!(response, {
    user: {
        profile: {
            settings.notifications.email: true,
            age: >= 18,
        }
    },
    items: len() > 0,
});
```

## Development Commands

### Build & Test
```bash
cargo build                    # Build in debug mode
cargo test                     # Run all tests
cargo test -- --nocapture     # Run tests with println! output
cargo test <test_name>        # Run specific test
cargo test --doc              # Run documentation tests
```

### Code Quality
```bash
cargo fmt                      # Format code
cargo fmt -- --check          # Check formatting
cargo clippy                  # Run linter
cargo clippy -- -D warnings   # Strict linting
```

### Documentation
```bash
cargo doc --open              # Build and view documentation
cargo test --doc              # Test documentation examples
```

### Procedural Macro Development
```bash
cargo expand --test basic     # Expand macros in test file (requires cargo-expand)
RUST_BACKTRACE=1 cargo test  # Debug macro panics
```

## Architecture

### Project Structure
- **src/lib.rs**: Main entry point, exports the `assert_struct!` macro
- **src/lib.rs** (procedural macro crate): Will contain the macro implementation
  - Token parsing and syntax validation
  - AST generation for assertions
  - Error message formatting
- **tests/**: Integration tests demonstrating all features
  - Each test file should focus on a specific feature set
  - Include both success and failure cases

### Implementation Strategy

1. **Macro Input Parsing**: Parse the expected structure syntax into an AST
2. **Code Generation**: Generate appropriate assertion code for each field
3. **Error Handling**: Produce clear, helpful error messages on mismatches
4. **Extensibility**: Design matcher trait system for custom validators

### Key Design Decisions

- **Procedural macro** (not declarative) for maximum flexibility
- **Compile-time validation** where possible
- **Zero runtime overhead** - expand to direct field access
- **Progressive enhancement** - start simple, add features incrementally

## Documentation Standards

- Every public API must have doc comments with examples
- Examples should show both successful and failing cases
- Use `cargo test --doc` to ensure all examples compile and run
- Include "why" documentation for complex matcher syntax

## Testing Strategy

- Unit tests for macro parsing logic
- Integration tests for each feature
- Doc tests for all public APIs
- Failure case tests to ensure good error messages
- Complex nested structure tests mimicking real JSON responses