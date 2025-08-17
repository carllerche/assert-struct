# Advanced Expression Patterns Design

## Overview

This document explores design options for supporting advanced expressions in `assert_struct!` patterns, allowing assertions on computed values rather than just direct field comparisons.

## Current Limitations

```rust
// What works today
assert_struct!(user, User {
    name: "Alice",
    age: > 18,
    profile: Some(_),
    ..
});

// What requires separate assertions
assert_eq!(user.profile.unwrap().settings.len(), 3);
assert!(user.name.to_lowercase().starts_with("a"));
assert!(matches!(user.status, Status::Active | Status::Premium));
```

## Design Option 1: Method Call Syntax

### Syntax
```rust
assert_struct!(user, User {
    name.len(): > 5,
    name.to_lowercase(): "alice",
    profile.is_some(): true,
    profile.unwrap().settings.len(): 3,
    tags.contains("admin"): true,
    ..
});
```

### Parsing Strategy
- Extend field parsing to detect `.method()` chains
- Parse method calls as part of field path: `field.method1().method2(args)`
- Support both no-arg and simple literal arg methods

### Code Generation
```rust
// For: name.len(): > 5
// Generate:
let __method_result_0 = (&value.name).len();
if !(__method_result_0 > 5) {
    // Error with method call context
}
```

### Pros
- **Natural Rust syntax** - looks like normal method calls
- **Discoverable** - IDE completion works for methods
- **Performant** - direct method calls, no closure overhead
- **Type-safe** - methods must exist at compile time
- **Good error messages** - can show exact method call that failed

### Cons
- **Limited flexibility** - only supports method chaining, not arbitrary expressions
- **Argument limitations** - complex arguments are hard to parse in this context
- **No control flow** - can't do `if`, `match`, loops, etc.
- **Parsing complexity** - need to handle method calls, generics, arguments
- **Tuple awkwardness** - `0.len(): 5` looks weird

### Implementation Challenges
- **Parsing method arguments** - especially complex expressions
- **Generic method calls** - `vec.into_iter().collect::<Vec<_>>()`
- **Disambiguation** - is `field.0` a method call or tuple access?
- **Error spans** - pointing to the right part of a method chain

## Design Option 2: Closure Syntax

### Syntax
```rust
assert_struct!(user, User {
    name: |n| n.len() > 5,
    name: |n| n.to_lowercase() == "alice", 
    profile: |p| p.is_some(),
    profile: |p| p.unwrap().settings.len() == 3,
    tags: |t| t.contains("admin"),
    status: |s| matches!(s, Status::Active | Status::Premium),
    data: |d| {
        match d.kind {
            DataKind::Simple => d.value < 100,
            DataKind::Complex => d.nested.len() > 0,
        }
    },
    ..
});
```

### Parsing Strategy
- Detect closure syntax: `|param| expr` or `|param| { block }`
- Parse as standard Rust closure using syn's `ExprClosure`
- No special parsing rules needed - leverage existing Rust parser

### Code Generation
```rust
// For: name: |n| n.len() > 5
// Generate:
let __closure_result_0 = (|n| n.len() > 5)(&value.name);
if !__closure_result_0 {
    // Error with closure context
}
```

### Pros
- **Maximum flexibility** - full Rust expressions supported
- **Familiar syntax** - standard Rust closures
- **Powerful patterns** - `match`, `if`, loops, complex logic
- **Works everywhere** - structs, tuples, enums equally well
- **Simple parsing** - reuse syn's existing closure parsing
- **Composable** - can call other functions, use external state

### Cons
- **Verbose** - `|n| n.len() > 5` vs `name.len(): > 5`
- **Runtime overhead** - closure calls vs direct method calls
- **Less discoverable** - no IDE completion inside closures
- **Harder error messages** - can't point to specific parts of closure
- **Debugging complexity** - harder to debug failed closures
- **Potential confusion** - mixing patterns with executable code

### Implementation Challenges
- **Error context** - how to show what went wrong inside a closure?
- **Performance** - closure call overhead for simple cases
- **Type inference** - ensuring parameter types are correct
- **Capture semantics** - what can closures capture from outer scope?

## Hybrid Approach: Both Syntaxes

### Concept
Support both syntaxes for different use cases:

```rust
assert_struct!(user, User {
    // Simple method calls - concise syntax
    name.len(): > 5,
    profile.is_some(): true,
    
    // Complex expressions - closure syntax
    status: |s| matches!(s, Status::Active | Status::Premium),
    data: |d| {
        match d.kind {
            DataKind::Simple => d.value < 100,
            DataKind::Complex => d.nested.len() > 0,
        }
    },
    ..
});
```

### Decision Tree
- **Simple method chains** → use method call syntax
- **Complex logic, control flow** → use closure syntax
- **Arguments beyond literals** → use closure syntax

## Comparison Matrix

| Feature | Method Call Syntax | Closure Syntax | Both |
|---------|-------------------|----------------|------|
| **Simplicity (simple cases)** | ✅ Excellent | ⚠️ Verbose | ✅ Best of both |
| **Flexibility** | ❌ Limited | ✅ Unlimited | ✅ Flexible |
| **Performance** | ✅ Direct calls | ⚠️ Closure overhead | ⚠️ Mixed |
| **Readability** | ✅ Natural | ⚠️ Can be dense | ✅ Context-appropriate |
| **Error messages** | ✅ Precise | ❌ Generic | ⚠️ Inconsistent |
| **Implementation complexity** | ⚠️ Custom parsing | ✅ Reuse syn | ❌ Both systems |
| **IDE support** | ✅ Full completion | ⚠️ Limited | ✅ Where applicable |
| **Learning curve** | ✅ Intuitive | ⚠️ New concept | ⚠️ Two syntaxes |

## Use Case Analysis

### AST Testing (Primary Driver)
```rust
// Method syntax - clean for simple cases
assert_struct!(function, ItemFn {
    sig.asyncness.is_some(): true,
    sig.inputs.len(): 2,
    sig.ident.to_string(): "test_func",
    ..
});

// Closure syntax - needed for complex checks
assert_struct!(function, ItemFn {
    sig.inputs: |inputs| {
        inputs.len() == 2 && 
        inputs.iter().all(|arg| matches!(arg, FnArg::Typed(_)))
    },
    attrs: |attrs| attrs.iter().any(|attr| attr.path.is_ident("test")),
    ..
});
```

### API Response Testing
```rust
assert_struct!(response, ApiResponse {
    // Method syntax for straightforward checks
    data.len(): > 0,
    errors.is_empty(): true,
    
    // Closure syntax for business logic
    data: |items| items.iter().all(|item| item.price > 0.0),
    metadata: |m| m.page_size * m.page_number <= m.total_count,
    ..
});
```

## Error Message Design

### Method Call Errors
```rust
assert_struct! failed:

   | User {
mismatch:
  --> `user.profile.unwrap().settings.len()` (line 15)
   |     profile.unwrap().settings.len(): > 0,
   |                                      ^^^ actual: 0 (method result)
   | }
```

### Closure Errors
```rust
assert_struct! failed:

   | User {
closure failed:
  --> `user.status` (line 18)
   |     status: |s| matches!(s, Status::Active | Status::Premium),
   |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ closure returned false
   |     actual value: Status::Inactive
   | }
```

## Implementation Unknowns

### Method Call Syntax
1. **Argument parsing boundaries** - where do method args end?
2. **Generic type inference** - how to handle `collect::<Vec<_>>()`?
3. **Macro hygiene** - variable naming in generated code
4. **Error recovery** - what if method doesn't exist?

### Closure Syntax
1. **Type parameter inference** - ensuring `|param|` gets right type
2. **Capture scope** - what variables can closures see?
3. **Error attribution** - mapping closure failures to source locations
4. **Performance characteristics** - overhead in hot paths

### Both Approaches
1. **Syntax disambiguation** - how to tell them apart during parsing?
2. **Error message consistency** - unified vs different styles
3. **Documentation complexity** - teaching both approaches
4. **Testing matrix** - exponential test combinations

## Design Decision: Closure Escape Hatch

**Implement closure syntax as an "escape hatch"** for cases where built-in patterns don't suffice.

### Core Principles

1. **Escape hatch philosophy** - Closures handle edge cases that patterns can't express
2. **Zero-cost abstraction** - Rust closures have no performance overhead anyway
3. **Test context** - Performance is less critical in test code
4. **Native Rust semantics** - Use standard Rust closure syntax and capture rules
5. **Single argument** - Closures take exactly one parameter (the field value)

### Technical Decisions

#### **Performance** ✅ Resolved
- Closures are zero-cost in Rust
- Test context makes performance less critical
- Acceptable trade-off for flexibility

#### **Verbosity** ✅ Resolved  
- `|x| x > 5` is acceptable verbosity for an escape hatch
- Primary patterns (`> 5`) handle common cases concisely
- Escape hatch can be more verbose

#### **Error Messages** ✅ Resolved
```rust
assert_struct! failed:

   | User {
closure mismatch:
  --> `user.age` (line 15)
   |     age: |x| x > 18 && x < 65,
   |          ^^^^^^^^^^^^^^^^^^^ closure returned false
   |     actual: 70
   | }
```

#### **Rust Semantics** ✅ Resolved
- Support full Rust closure syntax: `|x| expr`, `move |x| expr`, `|x| { block }`
- Use Rust's natural capture rules (by reference by default)
- `move` closures supported but expected to be rare

### Supported Syntax

```rust
assert_struct!(data, Data {
    // Basic closure
    value: |x| x > 5,
    
    // Block closure
    complex: |x| {
        match x.kind {
            Kind::A => x.value < 100,
            Kind::B => x.nested.len() > 0,
        }
    },
    
    // Move closure (captures by value)
    field: move |x| external_fn(x, captured_var),
    
    // Method chaining
    name: |n| n.to_lowercase().starts_with("test"),
    
    // Complex logic
    items: |items| {
        items.len() > 2 && 
        items.iter().all(|item| item.is_valid()) &&
        items.first().unwrap().priority > 0
    },
    ..
});
```

## Implementation Plan

### Phase 1: Core Closure Support
1. **Parsing** - Detect and parse `ExprClosure` in field patterns
2. **Code generation** - Call closure with field value, assert on boolean result  
3. **Error handling** - "closure mismatch" with field value display
4. **Basic testing** - Simple closure cases

### Phase 2: Full Rust Closure Features
1. **Move closures** - Support `move |x| expr` syntax
2. **Block closures** - Support `|x| { ... }` multiline blocks
3. **Complex expressions** - Method chaining, match expressions, etc.
4. **Comprehensive testing** - Edge cases and complex scenarios

### Phase 3: Polish & Documentation
1. **Error message refinement** - Better context and formatting
2. **Documentation** - Examples and best practices
3. **Integration testing** - Real-world usage patterns

## Parsing Implementation

```rust
// In parse.rs - detect closure syntax
if input.peek(Token![|]) || (input.peek(Token![move]) && input.peek2(Token![|])) {
    let closure: ExprClosure = input.parse()?;
    
    // Validate: exactly one parameter
    if closure.inputs.len() != 1 {
        return Err(Error::new(closure.span(), 
            "Closure must have exactly one parameter"));
    }
    
    return Ok(Pattern::Closure { closure });
}
```

## Code Generation

```rust
// For: age: |x| x > 18 && x < 65
// Generate:
let __closure = |x| x > 18 && x < 65;
let __closure_result = __closure(&value.age);
if !__closure_result {
    let __error = ErrorContext {
        field_path: "user.age".to_string(),
        pattern_str: "|x| x > 18 && x < 65".to_string(),
        actual_value: format!("{:?}", &value.age),
        error_type: ErrorType::Closure,
        // ...
    };
    __errors.push(__error);
}
```

---

**Next steps:** Prototype closure syntax implementation to validate assumptions and measure implementation complexity.