# TODO

## Next Features

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

### 3. Range support (Partially Implemented) ⚠️
- [x] Add range syntax recognition in parser
- [x] Add `Range` variant to `FieldAssertion` and `PatternElement`
- [x] Implement basic code generation for range assertions
- [ ] **Issue**: Reference handling needs architectural changes
  - The current destructuring with `&` creates reference fields
  - Range construction happens at parse time with literals
  - This creates type mismatches (`&u32` vs `Range<u32>`)
- [ ] Potential solutions:
  - Clone/dereference fields before range checking
  - Use a different assertion approach
  - Generate custom range checking code
- [ ] Test with all numeric types once working
- [ ] Update documentation

### 4. Improved regex operator
- [ ] Consider allowing variables containing regex patterns
- [ ] Consider function calls returning regex patterns
- [ ] Need to carefully design compile-time vs runtime behavior
- [ ] Maintain backward compatibility with current `=~ r"pattern"` syntax
- [ ] Document any limitations or trade-offs

## Architecture Notes

These features fit within the current design because:
- They all follow the "operator followed by expression" pattern
- They don't introduce parsing ambiguities
- They maintain the principle of "no arbitrary expressions in pattern positions"