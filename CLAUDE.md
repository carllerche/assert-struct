# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`assert-struct` is a procedural macro library for ergonomic structural assertions in Rust tests. It enables deep, partial matching of complex data structures without manually referencing every field - particularly useful for testing typed JSON responses and other nested data structures.

**Important:** This is a new, experimental crate in early development. We are NOT concerned with:
- Backward compatibility (we can make breaking changes freely)
- Migration guides (no existing users to migrate)
- Performance benchmarks (premature optimization)
- Semantic versioning constraints

Focus on clean design, good architecture, and comprehensive functionality rather than compatibility concerns.

### Design Philosophy

1. **Simplicity First**: Resist adding features just because they're possible. Ship minimal valuable functionality, then expand based on real user needs.
2. **No Speculative Features**: Don't implement "nice to have" features without clear use cases. Examples of avoided complexity:
   - Utility types (CaseInsensitive, Prefix, Suffix) - users can implement these if needed
   - Complex pattern combinators (Vec for OR, tuples for AND) - adds complexity without proven value
   - Option as wildcard pattern (using None to mean "any value") - clever but unnecessary
3. **Performance Optimization**: Only optimize the common case (string literals compiled at macro expansion time), not every possible case.

### Core Features (Implemented)
- **Partial matching**: Check only the fields you care about with `..`
- **Nested struct support**: Deep assertions without verbose field access chains
- **Nested field access**: Direct access to nested fields `outer.inner.field: value`
- **Comparison operators**: `<`, `<=`, `>`, `>=` for numeric assertions
- **Equality operators**: `==`, `!=` for explicit equality checks
- **Range patterns**: `18..=65`, `0.0..100.0` for range matching
- **Regex patterns**: `=~ r"pattern"` for string matching (feature-gated)
- **Like trait**: `=~ expr` for flexible pattern matching with variables/expressions
- **Slice patterns**: Element-wise patterns for Vec fields `[> 0, < 10, == 5]`
- **Enum support**: Full support for Option, Result, and custom enums (all variant types)
- **Tuple support**: Multi-field tuples with advanced patterns `(> 10, < 30)`
- **Method call patterns**: `field.method(): value` and `(0.method(): value, _)` for tuple elements
- **Pattern composition**: Combine all features (e.g., `Some(> 30)`, `Event::Click(>= 0, < 100)`)
- **Wildcard struct patterns**: Use `_` instead of type names to avoid imports `_ { field: value, .. }`
- **Smart pointer dereferencing**: Direct pattern matching through `Box`, `Arc`, `Rc` with `*field: value`

### Improved Error Messages
- **Fancy error formatting**: Shows pattern context with precise failure location
- **Pattern underlining**: Exact position of mismatch highlighted with `^^^^^`
- **Field path tracking**: Full path to failing field (e.g., `user.profile.age`)
- **Equality vs comparison**: Different formatting for `==` patterns showing expected value
- **Zero runtime cost**: Pattern strings generated at compile time

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
    items.len(): > 0,        // Method call patterns
    items: [> 0, < 100, > 0],  // Element-wise patterns
    ..
});

// With improved errors showing:
assert_struct! failed:

   | Response { ... Profile {
comparison mismatch:
  --> `response.user.profile.age` (line 10)
   |         age: >= 18,
   |              ^^^^^ actual: 17
   | } ... }
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

### Workspace Structure
```
assert-struct/
├── assert-struct/              # Main crate (provides Like trait, re-exports macro)
│   ├── src/
│   │   ├── lib.rs             # Like trait definition and implementations
│   │   └── error.rs           # NEW: Error formatting and fancy display
│   └── tests/                 # Integration tests
├── assert-struct-macros/       # Procedural macro crate
│   ├── src/
│   │   ├── lib.rs            # Macro entry point, Pattern enum
│   │   ├── parse.rs          # Token parsing and syntax validation
│   │   └── expand.rs         # Code generation (with pattern formatting)
│   └── Cargo.toml            # Has assert-struct as dev-dependency for testing
└── Cargo.toml                 # Workspace root
```

### Key Components
- **assert-struct-macros/src/lib.rs**: Defines core types
  - `AssertStruct`, `Expected`, `FieldAssertion` - core parsing structures
  - `Pattern` enum - unified abstraction for all pattern types
  - `ComparisonOp` - comparison operator types
- **assert-struct-macros/src/parse.rs**: Token parsing and syntax validation
  - `check_for_special_syntax` - disambiguates patterns from expressions
  - Dual-path for regex: `Pattern::Regex(String)` for literals, `Pattern::Like(Expr)` for expressions
- **assert-struct-macros/src/expand.rs**: Code generation
  - Generates match expressions for enums (not let bindings) for exhaustive matching
  - Handles recursive pattern expansion for nested structures
  - Optimizes string literal regexes at compile time
  - **NEW**: Generates `__PATTERN` const with formatted pattern string
  - **NEW**: Includes pattern location tracking for error messages
- **assert-struct/src/lib.rs**: Runtime support
  - `Like` trait for flexible pattern matching
  - Implementations for String/&str with regex patterns
- **assert-struct/src/error.rs**: NEW - Error formatting
  - `ErrorContext` struct with pattern location info
  - `format_fancy_error()` for rich error display
  - Pattern underlining and context display

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
- **Static pattern strings** - Generate formatted patterns at compile time for error messages

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
6. **Static pattern generation** - Pattern strings computed at compile time for error messages

### Critical Implementation Details
1. **Disambiguation is key**: The `check_for_special_syntax` function elegantly solves the ambiguity between patterns like `Some((true, false))` (simple expression) and `Some(> 30)` (comparison pattern)
2. **Fork and peek pattern**: Using `fork()` to look ahead without consuming tokens is essential for complex parsing
3. **Enum variant detection**: Checking if a path has multiple segments (e.g., `Status::Active` vs `Location`) helps distinguish enum variants from structs
4. **Special handling for Option/Result**: Custom logic for `Some`, `None`, `Ok`, `Err` provides better error messages
5. **Dual-path regex optimization**: String literals compile at macro expansion time (`Pattern::Regex`), while expressions use Like trait at runtime (`Pattern::Like`)
6. **Workspace circular dependency solution**: assert-struct-macros has assert-struct as a dev-dependency, allowing doc tests to work
7. **Pattern location tracking**: Runtime search in pattern string to find exact error position

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

### Key Learning: Leverage Rust's Native Syntax When Possible
When implementing proc macros, prefer generating code that uses Rust's native syntax rather than method calls:
- **Principle**: Rust's built-in syntax (patterns, match expressions, operators) handles many complexities automatically
- **Example**: Range matching
  ```rust
  // Instead of: if !(18..=65).contains(age) { panic!() }  // Requires managing types/references manually
  // Generate:  match age { 18..=65 => {}, _ => panic!() }  // Rust handles everything!
  ```
- **Benefits**:
  - Automatic reference level handling
  - Type inference works better
  - More idiomatic generated code
  - Leverages Rust compiler's full capabilities
- **Application**: Before implementing complex logic, ask "Can Rust's syntax already do this for us?"

### Key Learnings & Patterns

#### Workspace Organization
- **Dev-dependency pattern**: Procedural macro crates can depend on their parent crate as a dev-dependency for testing
- **Separation of concerns**: Runtime functionality (Like trait) in main crate, compile-time (macro) in separate crate

#### Feature Development  
- **Start minimal**: Ship core functionality first, add features based on real user needs
- **Avoid speculative features**: Don't add "nice to have" features without clear use cases
- **Question every addition**: Before adding a feature, ask "Is this solving a real problem?"

#### Performance Patterns
- **Optimize the common case**: String literal regexes compile at macro expansion time
- **Don't over-optimize**: Runtime compilation for expression patterns is acceptable

#### Communication
- **Precise terminology**: "Performance optimization" not "backward compatibility"
- **Clear project stage**: Early/experimental projects don't need migration guides or compatibility concerns

### Potential Future Extensions (Only If Needed)
- **Custom matcher functions**: `score: |s| s > 90 && s < 100` - only if users request it
- **HashMap/BTreeMap support**: Key-value matching - wait for use cases
- **Better error recovery**: More helpful messages for invalid macro syntax - as issues arise

## Current Work Status

### Recently Completed: Improved Error Messages (Phase 1)
- ✅ Static pattern string generation at compile time
- ✅ Enhanced ErrorContext with pattern location tracking  
- ✅ Fancy error formatter with pattern underlining
- ✅ Different error types (comparison, equality, range, etc.)
- ✅ Field path tracking through nested structures

### Potential Future Improvements
1. Multiple failure collection - show all failures at once instead of stopping at first
2. Slice diff format for literals - better visualization of array/vec differences
3. More accurate pattern location tracking - improved error underlining
4. Enhanced error details - regex pattern notes, Like trait custom messages