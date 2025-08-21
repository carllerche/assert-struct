# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-01-18

### Added

Initial release of assert-struct - a procedural macro for ergonomic structural assertions in Rust tests.

#### Core Features
- **Structural assertions** - Assert on struct fields with pattern matching syntax
- **Partial matching** - Use `..` to ignore fields you don't care about  
- **Nested structures** - Deep assertions without verbose field access chains
- **String literals** - Direct string comparison without `.to_string()`

#### Pattern Types
- **Comparison operators** - `>`, `<`, `>=`, `<=` for numeric comparisons
- **Equality operators** - `==`, `!=` for explicit equality checks
- **Range patterns** - `18..=65`, `0.0..100.0` for boundary checks
- **Regex patterns** - `=~ r"pattern"` for string matching (feature-gated)
- **Like trait** - Custom pattern matching via `Like<T>` trait
- **Wildcard patterns** - `_` to assert field exists without checking value

#### Data Type Support
- **Collections** - Element-wise patterns for `Vec` and slices
- **Tuples** - Full tuple support with advanced patterns
- **Enums** - `Option`, `Result`, and custom enum variants
- **Smart pointers** - Dereference `Box<T>`, `Rc<T>`, `Arc<T>` with `*field`

#### Advanced Features
- **Method calls** - `field.len(): 5`, `field.is_some(): true`
- **Field operations** - Dereferencing, method calls, nested field access
- **Index operations** - Support for `field[index]` patterns in nested structures
- **Repeated field patterns** - Multiple constraints on the same field (e.g., `age: >= 10, age: <= 99`)
- **Closure patterns** - Custom validation logic with closures
- **Pattern composition** - Combine multiple pattern types

#### Error Messages
- **Detailed error output** - Shows exact field path and location
- **Pattern context** - Visual representation of where failure occurred
- **Improved span handling** - Error messages point to specific tokens instead of entire macro calls
- **Multiple error types** - Specific messages for different failure modes
- **Zero runtime cost** - Error formatting only on failure

#### Documentation
- Comprehensive API documentation with examples
- Real-world examples in `examples/` directory
- Complete pattern reference in macro documentation
- Comprehensive test coverage - 350+ tests across 29 test files

[0.1.0]: https://github.com/carllerche/assert-struct/releases/tag/v0.1.0