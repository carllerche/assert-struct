# Like Trait - Generic Pattern Matching Support

## Overview

Extend `assert-struct` to support generic "like" matching on any types through a trait-based system. This will enable users to implement custom pattern matching for their own types while providing built-in implementations for common cases like regex matching.

## Motivation

Currently, `assert-struct` hardcodes pattern matching behaviors:
- Regex matching is limited to literal patterns (`=~ r"pattern"`)
- No way to extend matching behavior for custom types
- No way to use variables or expressions that resolve to patterns

By introducing a `Like` trait, we can:
1. Enable custom pattern matching for user-defined types
2. Support regex patterns from variables/expressions
3. Provide a clean, extensible API for pattern matching
4. Maintain backward compatibility with existing syntax

## Design

### The Like Trait

```rust
// In assert-struct crate (not macro crate)
pub trait Like<Rhs = Self> {
    fn like(&self, other: &Rhs) -> bool;
}
```

Based on `PartialEq` but for pattern matching rather than equality.

### Built-in Implementations

```rust
// String matches regex (from &str or String)
impl Like<&str> for String {
    fn like(&self, pattern: &&str) -> bool {
        // Compile pattern as regex and match against self
        regex::Regex::new(pattern)
            .map(|re| re.is_match(self))
            .unwrap_or(false)
    }
}

impl Like<String> for String {
    fn like(&self, pattern: &String) -> bool {
        self.like(&pattern.as_str())
    }
}

// Also implement for &str
impl Like<&str> for &str {
    fn like(&self, pattern: &&str) -> bool {
        regex::Regex::new(pattern)
            .map(|re| re.is_match(self))
            .unwrap_or(false)
    }
}

// Regex type for pre-compiled patterns
impl Like<regex::Regex> for String {
    fn like(&self, pattern: &regex::Regex) -> bool {
        pattern.is_match(self)
    }
}

impl Like<regex::Regex> for &str {
    fn like(&self, pattern: &regex::Regex) -> bool {
        pattern.is_match(self)
    }
}
```

### User Example

```rust
use assert_struct::{assert_struct, Like};

struct EmailAddress(String);

struct EmailPattern {
    domain: String,
}

impl Like<EmailPattern> for EmailAddress {
    fn like(&self, pattern: &EmailPattern) -> bool {
        self.0.ends_with(&format!("@{}", pattern.domain))
    }
}

// Usage
let email = EmailAddress("user@example.com".to_string());
let pattern = EmailPattern { domain: "example.com".to_string() };

assert_struct!(data, Data {
    email: =~ pattern,  // Use =~ operator for Like matching (same as regex)
    ..
});
```

## Implementation Plan

### Phase 1: Workspace Reorganization
- [x] Create workspace structure
- [ ] Move current crate to `assert-struct-macros`
- [ ] Create new `assert-struct` crate
- [ ] Set up workspace Cargo.toml
- [ ] Ensure all tests still pass

### Phase 2: Basic Like Trait
- [ ] Define `Like` trait in assert-struct crate
- [ ] Implement for String/&str with regex
- [ ] Add tests for Like implementations
- [ ] Document the trait

### Phase 3: Macro Integration
- [ ] Extend `=~` operator to work with any expression (not just string literals)
- [ ] Generate code using `Like::like()` for `=~` patterns
- [ ] Maintain backward compatibility for `=~ r"pattern"` syntax
- [ ] Add macro tests

### Phase 4: Advanced Implementations
- [ ] Support for `regex::Regex` type
- [ ] Consider other standard library types
- [ ] Performance optimizations (cache compiled regexes?)
- [ ] More examples and documentation

### Phase 5: Migration & Polish
- [ ] Update all documentation
- [ ] Migration guide for existing users
- [ ] Ensure smooth transition with clear examples
- [ ] Performance benchmarks

## Technical Considerations

### Workspace Structure
```
assert-struct/
├── Cargo.toml                 # Workspace root
├── assert-struct/              # Main crate (reexports macro)
│   ├── Cargo.toml
│   ├── src/
│   │   └── lib.rs             # Like trait, reexports
│   └── tests/                 # Integration tests
├── assert-struct-macros/       # Proc macro crate
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs             # Macro entry point
│       ├── parse.rs           # Parser
│       └── expand.rs          # Code generation
└── todo/
    ├── README.md
    └── like.md                # This file
```

### Backward Compatibility
- Keep `=~ r"pattern"` working (transforms to Like internally)
- Existing tests should pass without modification
- Clear migration path documented

### Performance
- Consider caching compiled regex patterns
- Benchmark Like trait overhead vs direct comparison
- Optimize common cases

### Error Messages
- Clear errors when Like trait not implemented
- Helpful suggestions for common mistakes
- Good compile-time diagnostics

## Open Questions

1. **Feature Gating**: Should Like trait be behind a feature flag initially?
2. **Default Implementations**: Should we provide blanket impls for some cases?
3. **Async Support**: Consider async pattern matching in the future?
4. **Pattern Types**: Support for glob patterns, SQL LIKE patterns, etc?
5. **Error Handling**: How to handle pattern compilation errors at runtime?

## Benefits

1. **Extensibility**: Users can define custom matching logic
2. **Type Safety**: Compile-time verification of pattern types
3. **Performance**: Pre-compiled patterns, optimized matching
4. **Flexibility**: Patterns from variables, functions, etc.
5. **Clean API**: Trait-based design follows Rust idioms

## Migration Example

Before:
```rust
assert_struct!(user, User {
    email: =~ r".*@example\.com",
    ..
});
```

After (both work):
```rust
// Still works (backward compatible)
assert_struct!(user, User {
    email: =~ r".*@example\.com",
    ..
});

// New way with variable (using same =~ operator)
let domain_pattern = regex::Regex::new(r".*@example\.com").unwrap();
assert_struct!(user, User {
    email: =~ domain_pattern,
    ..
});

// Or with custom type (still using =~)
let pattern = EmailDomainPattern::new("example.com");
assert_struct!(user, User {
    email: =~ pattern,
    ..
});
```

## Next Steps

1. Review and refine this plan
2. Create tracking issue on GitHub
3. Start with Phase 1 (workspace reorganization)
4. Implement incrementally with tests at each phase
5. Gather feedback from users