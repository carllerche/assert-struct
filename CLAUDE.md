# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`assert-struct` is a procedural macro library for ergonomic structural assertions in Rust tests. It enables deep, partial matching of complex data structures without manually referencing every field - particularly useful for testing typed JSON responses and other nested data structures.

### Core Features (Implemented)
- **Partial matching**: Check only the fields you care about with `..`
- **Nested struct support**: Deep assertions without verbose field access chains
- **Comparison operators**: `<`, `<=`, `>`, `>=` for numeric assertions
- **Regex patterns**: `=~ r"pattern"` for string matching (feature-gated)
- **Collection support**: Assert on Vec using slice syntax `[1, 2, 3]`
- **Enum support**: Full support for Option, Result, and custom enums (all variant types)
- **Tuple support**: Multi-field tuples with advanced patterns `(> 10, < 30)`
- **Pattern composition**: Combine all features (e.g., `Some(> 30)`, `Event::Click(>= 0, < 100)`)

### Example Use Case
```rust
// Instead of:
assert_eq!(response.user.profile.settings.notifications.email, true);
assert!(response.user.profile.age >= 18);
assert!(response.items.len() > 0);

// You can write:
assert_struct!(response, Response {
    user: User {
        profile: Profile {
            age: >= 18,
            ..
        },
        ..
    },
    items: [_, _, _],  // At least 3 items
    ..
});
```

## Development Commands

### Build & Test
```bash
cargo build                    # Build in debug mode
cargo test                     # Run all tests
cargo test --no-default-features  # Test without regex feature (IMPORTANT for CI)
cargo test -- --nocapture     # Run tests with println! output
cargo test <test_name>        # Run specific test
cargo test --doc              # Run documentation tests
```

### Code Quality
```bash
cargo fmt                                # Format code
cargo fmt -- --check                    # Check formatting
cargo clippy                            # Run linter
cargo clippy --all-targets -- -D warnings  # Strict linting (ALWAYS run before committing)
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
- **src/lib.rs**: Main entry point, defines core types and macro
  - `AssertStruct`, `Expected`, `FieldAssertion` - core parsing structures
  - `PatternElement` enum - unified abstraction for all pattern types
  - `ComparisonOp` - comparison operator types
- **src/parse.rs**: Token parsing and syntax validation
  - `parse_variant_tuple_contents` - handles special syntax like `Some(> 30)`
  - `check_for_special_syntax` - disambiguates patterns from expressions
  - `parse_tuple_elements` - recursive tuple pattern parsing
- **src/expand.rs**: Code generation
  - Generates match expressions for enums (not let bindings) for exhaustive matching
  - Handles recursive pattern expansion for nested structures
  - Transforms string literals to `.to_string()` automatically
- **tests/**: Integration tests demonstrating all features
  - `basic.rs` - fundamental struct matching
  - `comparison.rs` - comparison operators
  - `enums.rs` - Result, custom enums, tuple/struct variants
  - `option*.rs` - Option type tests (basic, advanced, nested)
  - `tuples.rs` - multi-field tuple support
  - `regex.rs` - regex pattern matching (feature-gated)

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

- Write tests BEFORE implementing features to clarify requirements
- Integration tests for each feature in separate files
- Doc tests for all public APIs with both success and failure examples
- Failure case tests with `#[should_panic]` to ensure good error messages
- Complex nested structure tests mimicking real JSON responses
- **ALWAYS test with `--no-default-features`** to ensure feature-gated code works
- **ALWAYS run `cargo clippy --all-targets -- -D warnings`** before committing
- Feature-gate tests using regex with `#[cfg(feature = "regex")]`

## Key Architectural Insights

### Design Strengths
1. **The `PatternElement` enum** is the heart of the macro - it elegantly unifies all pattern types
2. **Token-level parsing** gives maximum flexibility for special syntax like `Some(> 30)`
3. **Generating match expressions** (not let bindings) for enums ensures exhaustive matching
4. **Recursive pattern handling** makes deeply nested structures straightforward
5. **Zero runtime overhead** - everything expands to direct field access

### Critical Implementation Details
1. **Disambiguation is key**: The `check_for_special_syntax` function elegantly solves the ambiguity between patterns like `Some((true, false))` (simple expression) and `Some(> 30)` (comparison pattern)
2. **Fork and peek pattern**: Using `fork()` to look ahead without consuming tokens is essential for complex parsing
3. **Enum variant detection**: Checking if a path has multiple segments (e.g., `Status::Active` vs `Location`) helps distinguish enum variants from structs
4. **Special handling for Option/Result**: Custom logic for `Some`, `None`, `Ok`, `Err` provides better error messages

### Development Best Practices
1. **Incremental feature development**: Build features progressively (Option → Result → custom enums → tuples)
2. **Test-driven implementation**: Write comprehensive tests first to clarify requirements
3. **Document as you go**: Update documentation with each feature addition
4. **Commit after each major feature**: Creates clean history and allows easy rollback

### Common Pitfalls to Avoid
1. **CI considerations**: Always test with `--no-default-features` early
2. **Clippy on all targets**: Use `--all-targets` to catch issues in test code
3. **Feature gates**: Add `#[cfg(feature = "...")]` when first adding feature-dependent tests
4. **Dead code warnings**: Use `#[allow(dead_code)]` for fields/variants only used with certain features

### Future Extension Points
The architecture is well-positioned for these potential additions:
- **Range patterns**: `age: 18..=65`
- **Custom matcher functions**: `score: |s| s > 90 && s < 100`
- **HashMap/BTreeMap support**: Key-value matching
- **Performance optimization**: Cache compiled regex patterns
- **Better error recovery**: More helpful messages for invalid macro syntax