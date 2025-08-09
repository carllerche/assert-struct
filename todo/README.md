# TODO

## Active Development

### [Like Trait - Generic Pattern Matching](./like.md)
A major enhancement to support custom pattern matching through a trait-based system. This will enable:
- Custom pattern matching for user-defined types
- Regex patterns from variables/expressions  
- Clean, extensible API following Rust idioms
- See [like.md](./like.md) for detailed plan

## Completed Features

### 1. Equality operators (`==`, `!=`) ✅
- [x] Add `==` operator for explicit equality checks
  - Example: `age: == 30`
  - Example: `name: == "Alice"`
- [x] Add `!=` operator for inequality checks
  - Example: `status: != "error"`
  - Example: `count: != 0`
- [x] Support in all contexts (fields, tuples, enum variants)
- [x] Add comprehensive tests
- [x] Update documentation

### 2. Arbitrary expressions after operators ✅
- [x] Allow complex expressions after comparison operators
  - Example: `age: > compute_min_age()`
  - Example: `score: < get_threshold() + 10`
  - Example: `value: >= some_struct.field`
- [x] Ensure proper parsing of function calls, field access, method calls
- [x] Test with various expression types
- [x] Update documentation with examples

### 3. Range support ✅
- [x] Add range syntax recognition in parser
- [x] Add `Range` variant to `FieldAssertion`
- [x] Implement code generation using match expressions
- [x] **Solution**: Use Rust's pattern matching with ranges
  - Instead of `(18..=65).contains(age)`, generate `match age { 18..=65 => {}, _ => panic!() }`
  - This leverages Rust's built-in handling of reference levels in patterns
  - Works with all range types: `..=`, `..`, `n..`, `..n`, `..=n`
- [x] Test with all numeric types and chars
- [x] Update documentation
- **Note**: Full range `..` is intentionally not supported:
  - It's not a valid match pattern in Rust
  - It would be semantically confusing (different meaning than struct-level `..`)
  - No practical value (just omit the assertion or use struct-level `..`)

### 4. Slice patterns ✅
- [x] Add `SlicePattern` variant to `FieldAssertion`
- [x] Parse bracket syntax `[...]` for slice patterns
- [x] Support element-wise patterns in slices
  - Example: `[1, 2, 3]` for exact matching
  - Example: `[> 0, < 20, == 25]` with comparison operators
  - Example: `[=~ r"^alice", =~ r"^bob"]` with regex patterns
- [x] Generate proper assertions for each element
- [x] Add length checking with clear error messages
- [x] Test with various element types and patterns
- [x] Update documentation with slice examples

### 5. Enhanced slice patterns ✅
- [x] Support partial slice matching with `..`
  - Example: `[1, 2, ..]` to match first N elements
  - Example: `[.., 5]` to match last element
  - Example: `[1, .., 5]` to match first and last
- [x] Support slice patterns inside Option/Result
  - Example: `Some([1, 2, 3])`
- [x] Support empty slice syntax `[]`

## Future Ideas

### Improved Error Messages
- Include field paths in error messages (e.g., "user.profile.age: Value not in range")
- Show expected vs actual values more clearly
- Better diagnostics for complex nested structures

### HashMap/BTreeMap Support
- Pattern matching for map types
- Key-value assertions
- Example: `map: { "key1": > 10, "key2": == 20 }`

### Length Assertions
- Assert on collection lengths
- Example: `items: len(3)` or `items: len(> 5)`
- Could be part of slice pattern syntax

### Custom Matcher Functions (Post-Like trait)
- Allow closures as matchers
- Example: `score: |s| s > 90 && s < 100`
- Would build on Like trait infrastructure

### Performance Optimizations
- Cache compiled regex patterns
- Optimize code generation for common patterns
- Benchmark and profile macro expansion

## Architecture Notes

These features fit within the current design because:
- They all follow the "operator followed by expression" pattern
- They don't introduce parsing ambiguities
- They maintain the principle of "no arbitrary expressions in pattern positions"