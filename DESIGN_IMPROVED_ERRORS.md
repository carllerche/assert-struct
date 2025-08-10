# Design Document: Improved Error Messages for assert-struct

## Executive Summary

This document outlines the design and implementation strategy for enhancing error messages in the `assert-struct` macro. The current implementation produces generic panic messages that lack context about which field failed and where in the structure the mismatch occurred. The new implementation will provide rich, contextual error messages with precise field paths, pattern visualization, and helpful diagnostics.

## Motivation

Current error messages are inadequate for debugging test failures:
```
thread 'test_name' panicked at tests/test.rs:17:5:
assertion `left == right` failed
  left: "Alice"
 right: "Bob"
```

This lacks critical information:
- Which field failed (`user.name`)?
- Where in the pattern did it fail?
- What was the full structural context?

## Goals

1. **Precise field identification**: Show exact path to failed field (e.g., `user.profile.age`)
2. **Pattern context**: Display the pattern with the failure point highlighted
3. **Helpful diagnostics**: Show actual vs expected values with clear formatting
4. **Multiple failures**: Collect and display all failures, not just the first
5. **Maintainable implementation**: Incremental changes that keep tests passing

## Design Overview

### Core Concept: Zero-Cost Error Context

Instead of generating simple `assert!()` or `panic!()` calls, we'll generate code that:
1. Performs assertions with zero overhead on success
2. Builds field paths ONLY when assertions fail
3. Captures pattern information at compile time
4. Formats rich error messages before panicking

### Key Design Principle: Pay Only On Failure

The critical insight is that we can defer ALL error message construction to the failure path:
- **Success path**: Just the assertion check, no string building
- **Failure path**: Build paths, format messages, then panic
- **String literals**: Let Rust compiler handle deduplication

### Key Components

1. **Lazy Path Building**: Use const arrays of string literals, join only on failure
2. **Pattern Metadata**: Preserve pattern source for error display
3. **Error Collection**: Gather failure information before panicking
4. **Message Formatting**: Generate readable, aligned error output

## Implementation Strategy

### Phase 1: Infrastructure (Keep existing tests passing)

#### 1.1 Error Context Type
```rust
// In assert-struct-macros/src/lib.rs
#[derive(Debug)]
struct ErrorContext {
    field_path: String,      // e.g., "user.profile.age"
    pattern_str: String,      // e.g., ">= 18"
    actual_value: String,     // e.g., "17"
    line_number: u32,        // From line!() macro with proper spanning
    file_name: &'static str, // From file!() macro with proper spanning
    error_type: ErrorType,    // Comparison, Range, Regex, etc.
}

enum ErrorType {
    Comparison,
    Range,
    Regex,
    Value,
    EnumVariant,
    // ...
}
```

#### 1.2 Zero-Cost Path Building
Build paths only on assertion failure using const string literals:
```rust
// Generated code - path built only on failure:
if !(user.profile.age >= 18) {
    // String literals are automatically handled by rustc
    const PATH: &[&str] = &["user", "profile", "age"];
    let field_path = PATH.join(".");

    let error = ErrorContext {
        field_path,
        pattern_str: ">= 18".to_string(),
        actual_value: format!("{:?}", user.profile.age),
        // ...
    };
    panic!("{}", format_error(error));
}
```

### Phase 2: Gradual Error Enhancement

#### 2.1 Modify Code Generation (Backwards Compatible)

Current generation:
```rust
assert_eq!(actual.field, expected, "assertion failed");
```

New generation with fallback:
```rust
if actual.field != expected {
    // New error handling
    let __error = ErrorContext {
        field_path: __path.clone(),
        pattern_str: stringify!(expected),
        actual_value: format!("{:?}", actual.field),
        // ...
    };
    panic!("{}", format_error(__error));
} else {
    // Success - no change needed
}
```

#### 2.2 Incremental Pattern Support

Start with simple patterns and gradually add support:
1. **Equality**: `field: "value"` → Basic implementation
2. **Comparisons**: `field: >= 18` → Add comparison context
3. **Ranges**: `field: 18..=65` → Add range formatting
4. **Nested structs**: Track path through nesting
5. **Enums**: Handle variant mismatches
6. **Slices**: Special handling for element indices
7. **Regex**: Include pattern in error message

### Phase 3: Error Message Formatting

#### 3.1 Single Failure Format
```rust
fn format_error(ctx: ErrorContext) -> String {
    format!(
        r#"assert_struct! failed:

   | User {{
comparison mismatch:
  --> `{}` (line {})
   |     age: >= 18,
   |          ^^^^^ actual: {}
   |     ..
   | }}"#,
        ctx.field_path,
        ctx.line_number.unwrap_or(0),
        ctx.actual_value
    )
}
```

#### 3.2 Multiple Failures
Collect all errors before panicking:
```rust
// Generated code structure:
let mut __errors = Vec::new();

// For each field check:
if !condition {
    __errors.push(ErrorContext { ... });
}

// After all checks:
if !__errors.is_empty() {
    panic!("{}", format_multiple_errors(__errors));
}
```

## Detailed Implementation Plan

### Step 1: Add Error Context Infrastructure (No visible changes)
1. Add `ErrorContext` struct and `ErrorType` enum
2. Add basic `format_error` function that produces current-style messages
3. Modify expansion to use new infrastructure but maintain current output
4. **Tests remain green**

### Step 2: Enhance Path Tracking
1. Modify `expand_struct` to thread path through recursive calls
2. Add path parameter to expansion functions
3. Generate path-building code
4. Update `format_error` to include paths in messages
5. **Tests now show improved paths in errors**

### Step 3: Pattern Source Preservation
1. Capture pattern tokens as strings during parsing
2. Store pattern representations in expansion context
3. Include pattern source in error messages
4. **Tests show patterns in error output**

### Step 4: Improve Comparison Patterns
1. Special handling for `<`, `<=`, `>`, `>=`
2. Show "actual: X" with operator context
3. **Comparison tests show enhanced errors**

### Step 5: Improve Range Patterns
1. Special format for range mismatches
2. Show range and actual value clearly
3. **Range tests show enhanced errors**

### Step 6: Handle Nested Structures
1. Track nesting depth for context abbreviation
2. Show `User { ... Profile {` style context
3. **Nested struct tests show improved context**

### Step 7: Enum Variant Errors
1. Detect variant mismatches vs field mismatches
2. Special formatting for "expected Some, got None"
3. **Enum tests show enhanced errors**

### Step 8: Slice Pattern Enhancements
1. Track element indices in slices
2. Special handling for multiple element failures
3. Diff view for literal arrays
4. **Slice tests show enhanced errors**

### Step 9: Regex Pattern Support
1. Include regex pattern in error message
2. Show "pattern failed to match" clearly
3. **Regex tests show enhanced errors**

### Step 10: Polish and Optimization
1. Implement path truncation for very long paths
2. Optimize string building
3. Handle edge cases

## Line Number Capture Strategy

### The Challenge
We need to capture the line number where each pattern appears in the original `assert_struct!` invocation, not where the generated code panics. This is especially tricky when collecting multiple errors.

### Solution: Use Built-in `line!()` with Proper Spanning

The key insight: if we use `quote_spanned!` correctly, the built-in `line!()` and `file!()` macros will report the original source location, not the generated code location!

#### Library-side support (in assert-struct/src/lib.rs):
```rust
// Hidden module for macro support functions
#[doc(hidden)]
pub mod __macro_support {
    // Just the formatting functions - no location capture needed!
    pub fn format_error(error: ErrorContext) -> String {
        // ... formatting logic
    }
    
    pub fn format_multiple_errors(errors: Vec<ErrorContext>) -> String {
        // ... formatting logic
    }
}
```

#### Step 1: Generated code with spanned line!() macro
```rust
// In expand.rs, when generating code:
let span = pattern.span;  // Original span from parsing

quote_spanned! {span=>
    {
        let __actual = &user;
        let __actual_field = &__actual.age;

        if !(__actual_field >= &18) {
            // line!() will report the original line thanks to quote_spanned!
            let __line = line!();
            let __file = file!();
            
            const __PATH: &[&str] = &["user", "age"];
            let __error = ErrorContext {
                field_path: __PATH.join("."),
                pattern_str: ">= 18".to_string(),
                actual_value: format!("{:?}", __actual_field),
                line_number: __line,
                file_name: __file,
                error_type: ErrorType::Comparison,
            };
            panic!("{}", ::assert_struct::__macro_support::format_error(__error));
        }
    }
}
```

#### Step 2: For multiple error collection
```rust
// When collecting multiple errors:
quote_spanned! {span=>
    {
        let mut __errors = Vec::new();

        // For each field check (each with its own span):
        {
            if !(/* condition */) {
                // Each pattern gets its own line number from its span
                let __line = line!();
                let __file = file!();
                
                __errors.push(ErrorContext {
                    line_number: __line,
                    file_name: __file,
                    // ... other fields
                });
            }
        }

        // After all checks:
        if !__errors.is_empty() {
            panic!("{}", ::assert_struct::__macro_support::format_multiple_errors(__errors));
        }
    }
}
```

### Implementation: Span Preservation for Line Numbers

We need to preserve spans during parsing so `line!()` and `file!()` report the correct location:

```rust
// In parse.rs - preserve spans during parsing
impl Parse for FieldAssertion {
    fn parse(input: ParseStream) -> Result<Self> {
        let field_name = input.parse::<Ident>()?;
        let field_span = field_name.span();  // Capture span
        // ... rest of parsing

        Ok(FieldAssertion {
            field_name,
            span: field_span,  // Store for later use
            // ...
        })
    }
}

// In expand.rs - use span for line number
fn expand_assertion(assertion: &FieldAssertion) -> TokenStream {
    let span = assertion.span;

    quote_spanned! {span=>
        // quote_spanned! ensures generated code has the original span
        // So line!() and file!() will report the correct location
        if !(/* condition */) {
            let __line = line!();  // Reports original line number
            let __file = file!();  // Reports original file name
            // ... build error
        }
    }
}
```

## Code Generation Examples

### Current Generation
```rust
// Input: assert_struct!(user, User { age: >= 18, .. });

// Current output:
assert!(user.age >= 18, "assertion failed");
```

### New Generation (with line capture)
```rust
// Input: assert_struct!(user, User { age: >= 18, .. });

// New output (generated with quote_spanned!):
{
    let __actual = &user;
    let __actual_field = &__actual.age;

    if !(__actual_field >= &18) {
        // line!() and file!() report original location thanks to quote_spanned!
        let __line = line!();
        let __file = file!();
        
        // Path built only on failure - zero cost on success
        const __PATH: &[&str] = &["user", "age"];
        let __error = ErrorContext {
            field_path: __PATH.join("."),
            pattern_str: ">= 18".to_string(),
            actual_value: format!("{:?}", __actual_field),
            line_number: __line,
            file_name: __file,
            error_type: ErrorType::Comparison,
        };
        panic!("{}", ::assert_struct::__macro_support::format_error(__error));
    }
}
```

## Error Message Specifications

The complete error message specifications with all examples are documented in [`ERRORS.md`](./ERRORS.md). That document contains:

- **17 detailed examples** covering all pattern types and edge cases
- **Exact formatting specifications** for each error type
- **Multi-failure scenarios** with specific formatting rules
- **Edge cases** like very long paths, large slices, etc.

### Key Examples from ERRORS.md

For quick reference, here are the main categories of errors we need to handle:

1. **Basic field mismatch** (Example 1) - Simple string equality
2. **Expression patterns** (Example 2) - Variables and expressions
3. **Nested structures** (Examples 3, 9) - Deep field paths
4. **Enum variants** (Example 4) - Some/None, Ok/Err, custom enums
5. **Slice patterns** (Examples 5, 8, 10, 11) - Element failures and diffs
6. **Range patterns** (Example 6) - Range boundary violations
7. **Option/Result with patterns** (Examples 7, 14) - Nested pattern matching
8. **Regex patterns** (Example 12) - Pattern match failures
9. **Tuple patterns** (Example 13) - Multi-element tuples
10. **Like trait patterns** (Example 15) - Custom matchers
11. **Multiple field failures** (Example 16) - Collecting all errors
12. **Long path truncation** (Example 17) - Path abbreviation rules

### Error Format Patterns

From ERRORS.md, we have these consistent formatting patterns:

```
assert_struct! failed:

   | StructName { ... NestedStruct {
<error_type> mismatch:
  --> `full.path.to.field` (line XXX)
   |     field_name: <pattern>,
   |                  ^^^^^^^ actual: <value>
   |                         [expected: <value>]  // Only for expressions
   |                         [failed: X op Y]     // Only for comparisons
   |     ..
   | } ... }
```

Refer to ERRORS.md for the complete specifications during implementation.

## Testing Strategy

1. **Maintain backwards compatibility**: Existing tests should continue to pass during initial phases
2. **Add error message tests**: New tests specifically for error message format
3. **Incremental validation**: Test each enhancement as it's added
4. **Snapshot testing**: Consider using snapshot tests for error message format

## Migration Path

1. **Phase 1-3**: Internal changes only, tests pass as-is
2. **Phase 4+**: Error messages improve incrementally
3. **No breaking changes**: Users see better errors automatically
4. **Feature flag option**: Could add `enhanced_errors` feature flag if needed

## Performance Considerations

1. **Compile time**: Minimal impact - just generating const arrays
2. **Runtime**: Zero overhead on success path - all path building happens only on failure
3. **Binary size**: Slightly larger due to error formatting code (error path only)
4. **String deduplication**: Rust compiler handles deduplication of string literals

## Future Enhancements

1. **Configurable verbosity**: Environment variable for error detail level
2. **JSON output option**: Machine-readable error format for tooling
3. **Color support**: Terminal colors for better readability
4. **Custom error handlers**: User-provided error formatting functions

## Success Criteria

1. All existing tests pass without modification
2. Error messages clearly identify failed fields
3. Pattern context is visible in errors
4. Implementation is maintainable and extensible
5. Performance impact is negligible

## Timeline Estimate

- Phase 1 (Infrastructure): 2-3 hours
- Phase 2 (Path tracking): 2-3 hours
- Phase 3 (Pattern preservation): 2-3 hours
- Phase 4-9 (Pattern types): 1-2 hours each
- Phase 10 (Polish): 2-3 hours

Total: ~20-25 hours of implementation

## Conclusion

This design provides a clear path to dramatically improved error messages while maintaining backwards compatibility and allowing incremental implementation. The phased approach ensures we can validate each enhancement and maintain a working codebase throughout development.