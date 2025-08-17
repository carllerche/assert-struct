# Auto-Deref Pattern Matching Design

## Overview

This document outlines the design for automatic dereferencing in `assert_struct!` patterns. This feature will allow transparent pattern matching through any type that implements `Deref`, including `Box<T>`, `Rc<T>`, `Arc<T>`, and custom smart pointers.

## Problem Statement

In AST parsing and other complex data structures, smart pointers are commonly used. Currently, users must either:

1. Use closure escape hatch for dereferencing  
2. Write separate assertions outside of `assert_struct!`

```rust
// Current workarounds:
assert_struct!(node, AstNode {
    // Option 1: Verbose closure
    expr: |boxed| matches!(**boxed, Expr::Literal(_)),
    
    // Option 2: Separate assertion (outside assert_struct)
    ..
});
assert!(matches!(**node.expr, Expr::Literal(_)));
```

## Key Insight: Rust's Match Behavior

**Critical Discovery**: Rust's `match` statements do NOT auto-dereference smart pointers:

```rust
let boxed: Box<Option<u32>> = Box::new(Some(42));

// ❌ This fails - no auto-deref through Box
match boxed {
    Some(x) => ...,  // Type error!
}

// ✅ This works - explicit deref
match *boxed {
    Some(x) => ...,  // Success!
}
```

## Proposed Solution

**Generate appropriate `*` operators** instead of special syntax:

```rust
// User writes natural patterns:
assert_struct!(node, AstNode {
    expr: Expr::Literal {
        value: > 0,
        ..
    },
    // Works with any Deref type!
    ..
});

// Macro detects field is Box<Expr> but pattern expects Expr
// Generates: match *node.expr { Expr::Literal { value, .. } => ... }
```

## Design Principles

1. **No Special Syntax**: Users write natural patterns, macro handles dereferencing
2. **Universal Deref Support**: Works with ANY type implementing `Deref`
3. **Type-Driven**: Let Rust's type system guide when dereferencing is needed
4. **Zero Runtime Cost**: Just generates `*` operators, no wrapper logic
5. **Composability**: Works with all existing patterns seamlessly

## How It Works

### Type-Driven Deref Detection

The macro analyzes the **field type** vs **pattern expectation**:

```rust
struct AstNode {
    expr: Box<Expr>,        // Field type: Box<Expr>
    maybe: Option<Rc<Data>>, // Field type: Option<Rc<Data>>
}

assert_struct!(node, AstNode {
    expr: Expr::Literal { .. },    // Pattern expects: Expr
    maybe: Some(Data { .. }),       // Pattern expects: Option<Data>
});
```

### Deref Chain Calculation

The macro calculates the "deref distance":

- `Box<Expr>` → `Expr` = 1 deref (`*field`)
- `Rc<Option<T>>` → `Option<T>` = 1 deref (`*field`) 
- `Arc<Box<T>>` → `T` = 2 derefs (`**field`)

### Integration with Existing Patterns

Auto-deref works transparently with ALL existing patterns:

```rust
assert_struct!(data, DataStruct {
    // With comparisons - Box<i32> auto-derefs to i32
    boxed_num: > 42,
    
    // With ranges - Box<u32> auto-derefs to u32  
    boxed_age: 18..=65,
    
    // With regex - Box<String> auto-derefs to String
    boxed_name: =~ r"^[A-Z][a-z]+$",
    
    // With wildcards - Rc<Option<T>> auto-derefs to Option<T>
    maybe_boxed: Some(_),
    
    // With structs - Box<User> auto-derefs to User
    boxed_user: User {
        name: "Alice",
        age: > 18,
        ..
    },
    
    // With enums - Arc<Result<T,E>> auto-derefs to Result<T,E>
    boxed_result: Ok {
        value: > 0,
        ..
    },
    
    // With tuples - Box<(i32,i32)> auto-derefs to (i32,i32)
    boxed_point: (> 0, < 100),
    
    // With slices/vectors - Box<Vec<T>> auto-derefs to Vec<T>
    boxed_vec: [1, 2, 3],
    
    // Nested smart pointers - Box<Rc<Option<String>>> auto-derefs as needed
    nested: Some("hello"),  // Box<Rc<Option<String>>> -> Option<String>
    
    // With closures (escape hatch still works)
    complex: |inner| inner.custom_validation(),  // Deref happens before closure
});

// Generated code examples:
// boxed_num: > 42        → match *data.boxed_num { x if x > 42 => ... }
// boxed_user: User { .. } → match *data.boxed_user { User { .. } => ... }
// nested: Some("hello")   → match **data.nested { Some("hello") => ... }
```

## Implementation Strategy

### 1. No AST Changes Needed!

The beauty of this approach: **no new pattern variants required**. We use existing patterns and generate appropriate dereferencing.

### 2. Type-Guided Code Generation

Instead of parsing special syntax, we analyze types during code generation:

```rust
// In expand.rs - modify field assertion generation
fn generate_field_assertion(field_type: &syn::Type, pattern: &Pattern, field_expr: &TokenStream) -> TokenStream {
    let deref_count = calculate_deref_distance(field_type, pattern);
    let deref_expr = apply_derefs(field_expr, deref_count);
    
    // Generate normal pattern matching with dereferenced expression
    generate_pattern_match(deref_expr, pattern)
}

fn calculate_deref_distance(field_type: &syn::Type, pattern: &Pattern) -> usize {
    // Analyze field type and pattern to determine how many * operators needed
    // This is where the magic happens!
}

fn apply_derefs(expr: &TokenStream, count: usize) -> TokenStream {
    match count {
        0 => quote! { #expr },
        1 => quote! { *#expr },
        2 => quote! { **#expr },
        n => {
            let stars = "*".repeat(n);
            let stars_tokens = proc_macro2::TokenStream::from_str(&stars).unwrap();
            quote! { #stars_tokens #expr }
        }
    }
}
```

### 3. Type Analysis Strategy

The core challenge: **determine when dereferencing is needed**:

```rust
fn calculate_deref_distance(field_type: &syn::Type, pattern: &Pattern) -> usize {
    match (field_type, pattern) {
        // Box<T> field with pattern expecting T
        (Type::Path(type_path), _) if is_box_type(type_path) => {
            let inner_type = extract_box_inner_type(type_path);
            if pattern_expects_type(pattern, &inner_type) {
                1  // Need one deref: Box<T> -> T
            } else {
                0  // Pattern matches Box itself
            }
        }
        
        // Rc<T> field with pattern expecting T  
        (Type::Path(type_path), _) if is_rc_type(type_path) => {
            // Similar logic...
            1
        }
        
        // Nested: Arc<Box<T>> field with pattern expecting T
        (Type::Path(type_path), _) if is_arc_type(type_path) => {
            let inner = extract_arc_inner_type(type_path);
            if is_box_type(&inner) {
                let box_inner = extract_box_inner_type(&inner);
                if pattern_expects_type(pattern, &box_inner) {
                    2  // Need two derefs: Arc<Box<T>> -> Box<T> -> T
                } else {
                    1  // Pattern expects Box<T>
                }
            } else {
                1  // Pattern expects inner type
            }
        }
        
        _ => 0  // No dereferencing needed
    }
}
```

### 4. Type Constraint Strategy

We need to ensure the smart pointer actually contains the expected type. This happens naturally through Rust's type system, but we should provide clear error messages.

### 5. Error Message Strategy

Extend error messages to show the dereferencing context:

```rust
// Error output:
assert_struct! failed:

   | AstNode {
pattern mismatch:
  --> `node.expr` (line 15)
   |     expr: Box<Expr::Literal {
   |           ^^^^^^^^^^^^^^^^^^^ expected Expr::Literal, found Expr::Binary
   |         value: > 0,
   |     }>,
   | }
```

## Edge Cases and Considerations

### 1. Multiple Dereference Levels

Support nested smart pointers:

```rust
// Should work:
nested: Box<Rc<Option<String>>>,
triple: Box<Rc<Arc<MyStruct { .. }>>>,
```

### 2. Generic Type Parameters

Handle generic smart pointers gracefully:

```rust
// This might be complex:
generic_box: Box<T> where T: SomePattern,
```

**Decision**: Start with concrete types only, add generic support later if needed.

### 3. Custom Deref Types

Should we support custom types that implement `Deref`?

**Decision**: Start with std library types (`Box`, `Rc`, `Arc`), add custom support if requested.

### 4. Reference Patterns

How does this interact with existing reference handling?

```rust
// These should be equivalent:
field: &Pattern,
field: Pattern,  // when field is already &T
```

**Decision**: Let Rust's type system handle this naturally.

### 5. Owned vs Borrowed

```rust
// Different scenarios:
owned_box: Box<Pattern>,        // We own the Box
borrowed_box: &Box<Pattern>,    // We borrow the Box
```

**Decision**: Generate appropriate dereferencing code based on context.

## Implementation Phases

### Phase 1: Core Auto-Deref Support
- Add type analysis to `expand.rs` for detecting smart pointer mismatches
- Implement `calculate_deref_distance()` for Box, Rc, Arc
- Generate appropriate `*` operators in field assertions
- Add basic tests for single-level dereferencing

### Phase 2: Multiple Smart Pointer Types & Nesting
- Support nested smart pointers (Box<Rc<T>>, Arc<Box<T>>, etc.)
- Handle reference vs owned scenarios properly
- Comprehensive testing for all combinations

### Phase 3: Integration & Polish
- Ensure all existing patterns work through auto-deref
- Error message improvements with deref context
- Performance testing and optimization
- Documentation and examples

### Phase 4: Advanced Cases
- Generic type parameter support (if needed)
- Custom Deref types (if requested by users)
- Edge case handling and robustness improvements

## Alternative Approaches Considered

### 1. Explicit Deref Operator

```rust
// Alternative syntax:
expr: *Box<Pattern>,
expr: **Rc<Option<Pattern>>,
```

**Rejected**: Less readable, doesn't follow Rust's natural container syntax.

### 2. Special Method Syntax

```rust
// Alternative syntax:
expr.deref(): Pattern,
expr.as_ref(): Pattern,
```

**Rejected**: Would require implementing method call patterns first, more complex.

### 3. Closure-Only Approach

```rust
// Alternative: Just use closures
expr: |boxed| matches!(**boxed, Pattern),
```

**Rejected**: Less ergonomic, doesn't provide structured pattern benefits.

## Testing Strategy

### Unit Tests
- Basic Box, Rc, Arc patterns
- Nested smart pointers
- Integration with all existing pattern types
- Error cases and edge conditions

### Integration Tests
- Real-world AST structures
- Complex nested scenarios
- Performance benchmarks

### Error Message Tests
- Snapshot tests for clear error output
- Various failure scenarios

## Future Extensions

### 1. Pin Support
```rust
pinned: Pin<Box<Future>>,
```

### 2. Custom Deref Types
Support for user-defined smart pointers.

### 3. Multiple Deref Chains
Automatically follow multiple `Deref` implementations.

## Summary

This design provides a natural, type-safe way to pattern match through smart pointers while maintaining `assert_struct!`'s core principles of clarity and performance. The phased implementation approach allows us to start with the most common cases and expand based on user feedback.

The key insight is that we can automatically generate appropriate `*` operators based on type analysis rather than requiring special syntax. This leverages Rust's existing match behavior, works with any `Deref` type, and provides maximum composability with existing features while keeping the implementation simple and performant.

## Implementation Status and Challenges

### Current State
- **Design**: Complete - comprehensive design document with clear approach
- **Runtime Support**: Added - `AutoDeref` trait and helper functions in `__macro_support`
- **Test Infrastructure**: Complete - test files with current closure workarounds
- **Macro Implementation**: Not yet implemented - significant challenges discovered

### Key Implementation Challenges Discovered

#### 1. Procedural Macro Limitations
**Problem**: Procedural macros operate at the syntax level without type information.
- Cannot determine at compile time whether a field is `Box<T>`, `Rc<T>`, or `T`
- Cannot safely attempt dereferencing without knowing the type supports `Deref`
- Type analysis requires the full Rust compiler's type checker

**Example Impact**:
```rust
// This breaks compilation if `value` is `Option<i32>` (doesn't implement Deref)
match *value {
    Some(x) => ...,  // Error: cannot dereference Option<i32>
}
```

#### 2. Universal Dereferencing Approach Failed
**Attempted Solution**: Generate fallback code that tries dereferencing all values
**Result**: Compile errors for non-`Deref` types like `Option<T>`, `Vec<T>`, etc.

**Learning**: Cannot use a "try dereferencing everything" approach - breaks existing functionality.

#### 3. Compile-Time Type Detection Challenge
**Core Issue**: Need to differentiate between:
- `field: Box<Option<i32>>` (needs `*field` to get `Option<i32>`)
- `field: Option<i32>` (direct pattern matching, no deref)
- `field: Rc<String>` (needs `*field` to get `String`)

**Macro Limitation**: All we see is the pattern `Some(42)` - we don't know the field type.

### Potential Solutions Under Research

#### 1. Trait-Based Runtime Detection
Generate code that uses trait bounds to conditionally enable auto-deref:
```rust
// Only compiles if T: Deref
fn try_auto_deref<T: std::ops::Deref>(value: &T) -> &T::Target { 
    value 
}
```

#### 2. Specific Smart Pointer Detection
Focus on known smart pointer patterns rather than universal approach:
- Detect `Box<Pattern>` syntax explicitly
- Generate specific deref code for known types
- Fail gracefully for unknown types

#### 3. User Annotation Approach
Add optional syntax for explicit auto-deref requests:
```rust
assert_struct!(test, TestStruct {
    boxed_field: *Some(42),  // Explicit deref request
    normal_field: Some(42),  // Normal matching
});
```

### Current Workaround: Closure Escape Hatch
The implemented closure feature provides an immediate solution:
```rust
assert_struct!(test, TestStruct {
    boxed_option: |b| matches!(**b, Some(42)),
    rc_string: |s| s.as_str() == "hello",
});
```

### Next Steps
1. **Research Phase**: Investigate compile-time type detection approaches
2. **Prototype**: Test trait-based conditional compilation approaches  
3. **Incremental**: Implement for specific smart pointer types first
4. **Validation**: Ensure no regression in existing functionality

The auto-deref feature remains a valuable goal, but implementation requires a more sophisticated approach than initially anticipated.