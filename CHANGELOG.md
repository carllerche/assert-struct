# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.1] - 2026-02-19

### Fixed
- **Method call regression** - Fixed E0716/E0507 errors when calling methods that return owned values or references (#105)

## [0.3.0] - 2026-02-19

### Added

#### Set Pattern Support
- **Set patterns** - New `#(pattern1, pattern2, ..)` syntax for unordered collection matching
- **Exact set matching** - `#(1, 2, 3)` requires the collection to contain exactly these elements
- **Partial set matching** - `#(> 0, < 10, ..)` finds elements matching each pattern, ignores the rest
- **Pattern flexibility** - All existing patterns work within set patterns (comparisons, ranges, regex, nested structs)
- **Empty sets** - `#()` matches exactly empty collections

### Fixed
- **Unreachable patterns warning** - Suppress spurious `unreachable_patterns` warnings for single-variant enums
- **Vec index assertions** - Fixed regression with index-based assertions on `Vec` fields (#93)

### Internal
- Refactored macro expansion for improved maintainability
- Consolidated comparison pattern parsing

## [0.2.0] - 2025-08-23

### Added

#### Map Pattern Support with Duck Typing
- **Map patterns** - New `#{ "key": pattern }` syntax for ergonomic map matching
- **Duck typing** - Works with any type implementing `len()` and `get(K) -> Option<&V>`
- **Pattern flexibility** - Supports HashMap, BTreeMap, and custom map types
- **Exact matching** - `#{ "key": "value" }` enforces exact map size
- **Partial matching** - `#{ "key": "value", .. }` ignores additional entries
- **Empty maps** - `#{}` matches exactly empty maps  
- **Wildcard maps** - `#{ .. }` matches any map regardless of contents
- **Rich patterns** - All existing patterns work in map values (comparisons, ranges, regex, nested structs)

#### Enhanced Await Support
- **Comprehensive await patterns** - Full support for `.await` in field expressions
- **Nested await chains** - Complex async expressions with multiple `.await` calls
- **Await with comparisons** - Combine async operations with pattern matching
- **Span propagation** - Accurate error locations for await expressions

#### Improved Error Reporting
- **Better span propagation** - Error messages point to specific syntax elements
- **Index operation spans** - Accurate error locations for array/slice indexing
- **Method call spans** - Precise error locations for method invocations
- **Async operation spans** - Proper error locations for await expressions

#### Development Experience
- **Re-exported regex types** - No need to directly depend on regex crate when using regex features
- **Compilation error tests** - Comprehensive trybuild tests for better error messages
- **Expanded test coverage** - 100+ additional tests covering new features

### Fixed
- **Type comparison issues** - Resolved `&usize` vs `usize` comparison errors in map patterns
- **Span propagation** - Fixed error locations pointing to entire macro instead of specific patterns
- **Regex dependencies** - Simplified regex feature usage by re-exporting types
- **Index operation errors** - Better error messages for invalid array/slice access patterns

### Technical Improvements  
- **Duck typing implementation** - Generic map support without hardcoded types
- **Enhanced parsing** - More robust handling of complex expressions
- **Better code generation** - Optimized macro expansion for new features
- **CI improvements** - Enhanced testing pipeline with compilation failure tests

## [0.1.0] - 2025-08-21

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

[0.3.1]: https://github.com/carllerche/assert-struct/releases/tag/v0.3.1
[0.3.0]: https://github.com/carllerche/assert-struct/releases/tag/v0.3.0
[0.2.0]: https://github.com/carllerche/assert-struct/releases/tag/v0.2.0
[0.1.0]: https://github.com/carllerche/assert-struct/releases/tag/v0.1.0