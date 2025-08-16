# assert-struct Roadmap

This document outlines planned features for assert-struct based on real-world usage analysis, particularly from analyzing AST parser tests that heavily use structural assertions.

## Currently Supported Features

Before listing missing features, it's important to note what's already available:

- ✅ **Basic struct matching** with partial matching via `..`
- ✅ **Enum matching** for specific variants (e.g., `Some(42)`, `None`, `Ok("value")`)
- ✅ **Empty collections** using `[]` syntax
- ✅ **Comparison operators** (`<`, `<=`, `>`, `>=`)
- ✅ **Equality operators** (`==`, `!=`)
- ✅ **Range patterns** (`18..=65`, `0.0..100.0`)
- ✅ **Regex patterns** (`=~ r"pattern"`)
- ✅ **Slice patterns** with element-wise matching (`[> 0, < 10, == 5]`)
- ✅ **Tuple patterns** including in enums
- ✅ **Nested struct matching** with deep field paths

## Priority 1: Critical Features for AST Testing

These features are essential for testing parser output and other deeply nested data structures.

### 1. Vec/Array Index Access Patterns
**Status:** Not Implemented  
**Use Case:** Asserting on specific elements in collections by index  
**Current Workaround:** Multiple separate assertions or manual match statements

```rust
// Proposed syntax:
assert_struct!(func, ItemFn {
    sig: Signature {
        inputs[0]: FnArg::Typed { 
            pat: Pat::Ident { ident.name: "x", .. },
            ..
        },
        inputs[1]: FnArg::Typed {
            pat: Pat::Ident { ident.name: "y", .. },
            ..
        },
        ..
    },
    ..
});
```

### 2. Method Call Assertions  
**Status:** Not Implemented  
**Use Case:** Asserting on method results like `is_some()`, `is_empty()`, `len()`  
**Current Workaround:** Separate assertions outside of assert_struct

```rust
// Proposed syntax:
assert_struct!(func, ItemFn {
    sig: Signature {
        asyncness.is_some(): true,
        inputs.is_empty(): false,
        generics.params.len(): 2,
        ..
    },
    ..
});
```

### 3. Wildcard Patterns in Enums and Structs
**Status:** Not Implemented  
**Use Case:** Checking enum variant without caring about its contents (e.g., `is_some()` checks)  
**Current Workaround:** Match statements that ignore the inner value or separate assertions

```rust
// Proposed syntax:
assert_struct!(data, MyStruct {
    // Check Option is Some without caring about value
    maybe_value: Some(_),
    
    // Check variant without caring about fields
    fields: Fields::Named(_),
    
    // Check Result is Ok without caring about value
    result: Ok(_),
    ..
});
```

**Note:** Currently `_` is not valid in expression position, so this requires special handling in the macro.

### 4. Box/Deref Pattern Matching
**Status:** Not Implemented  
**Use Case:** Matching through Box, Rc, Arc, and other smart pointers  
**Current Workaround:** Dereferencing in match statements

```rust
// Proposed syntax:
assert_struct!(ref_type, RefType {
    elem: Box<Type::Path { 
        path.segments[0].ident.name: "str",
        ..
    }>,
    ..
});
```

## Priority 2: Common Convenience Features

These features would significantly improve ergonomics for common test patterns.

### 5. Length/Count Assertions
**Status:** Not Implemented  
**Use Case:** Direct assertions on collection sizes  
**Current Workaround:** Separate `assert_eq!` for lengths

```rust
// Proposed syntax options:
assert_struct!(trait_item, ItemTrait {
    items.len(): 1,
    supertraits.len(): 2,
    ..
});

// Alternative syntax:
assert_struct!(trait_item, ItemTrait {
    items: [_; 1],  // Exactly 1 element, don't care what
    supertraits: [_, _],  // Exactly 2 elements
    ..
});
```

### 6. Empty Collection Assertions
**Status:** Already Supported ✓  
**Use Case:** Explicitly asserting collections are empty  

```rust
// Already works:
assert_struct!(func, ItemFn {
    sig: {
        inputs: [],  // Empty vec
        generics.params: [],
        ..
    },
    ..
});
```

### 7. Boolean Negation
**Status:** Not Implemented  
**Use Case:** Asserting boolean fields are false  
**Current Workaround:** Using `assert!(!value)`

```rust
// Proposed syntax:
assert_struct!(ptr_type, PtrType {
    const_token: false,  // or !true
    mutability: Mutability::Mut,
    ..
});
```

## Priority 3: Advanced Pattern Matching

These features enable more sophisticated assertions for complex scenarios.

### 8. Partial Vec/Slice Matching with Wildcards
**Status:** Not Implemented  
**Use Case:** Match some elements, ignore others  
**Current Workaround:** Individual element checks

```rust
// Proposed syntax:
assert_struct!(generics, Generics {
    params: [
        GenericParam::Lifetime { .. },
        _,  // Don't care about second param
        GenericParam::Const { ident.name: "N", .. },
    ],
    ..
});
```

**Note:** This depends on wildcard pattern support being implemented first.

### 9. Pattern Guards/Conditional Patterns
**Status:** Not Implemented  
**Use Case:** Additional conditions on patterns  
**Current Workaround:** Separate conditional assertions

```rust
// Proposed syntax:
assert_struct!(value, MyStruct {
    field: Some(x) if x > 10,
    name: s if s.starts_with("test_"),
    ..
});
```

### 10. Custom Matcher Functions
**Status:** Not Implemented  
**Use Case:** Arbitrary predicate functions for complex logic  
**Current Workaround:** Multiple assertions or custom validation code

```rust
// Proposed syntax:
assert_struct!(block, Block {
    stmts: |s| s.iter().all(|stmt| matches!(stmt, Stmt::Local(_))),
    ..
});
```

### 11. String Pattern Extensions
**Status:** Partially Implemented (regex only)  
**Use Case:** Common string matching beyond regex  
**Current Workaround:** Regex patterns or separate assertions

```rust
// Proposed syntax:
assert_struct!(ident, Ident {
    name: starts_with("test_"),
    // or
    name: ends_with("_impl"),
    // or  
    name: contains("async"),
    ..
});
```

## Implementation Strategy

### Phase 1: Foundation (Priority 1 Features)
1. **Vec/Array indexing** - Most critical for AST testing
2. **Method call assertions** - Enables is_some(), len(), etc.
3. **Wildcard patterns** - Essential for variant checking and is_some() style assertions
4. **Box/Deref patterns** - Common in AST structures

### Phase 2: Ergonomics (Priority 2 Features)
5. Length assertions (beyond just empty)
6. Boolean negation
7. Pattern guards and conditional patterns

### Phase 3: Advanced (Priority 3 Features)
8. Partial vec matching with wildcards (depends on wildcard support)
9. Custom matcher functions
10. String pattern extensions (starts_with, ends_with, contains)

## Design Principles

1. **Zero Runtime Cost**: All pattern analysis at compile time where possible
2. **Intuitive Syntax**: Patterns should feel like natural Rust
3. **Clear Error Messages**: Maintain high-quality error reporting
4. **Composability**: Features should work well together
5. **Backward Compatibility**: Don't break existing patterns

## Non-Goals

- Supporting every possible Rust pattern (keep it focused on testing)
- Runtime pattern compilation (except where necessary like dynamic regex)
- Becoming a general pattern matching framework

## Success Metrics

A successful implementation would allow expressing most AST parser tests as single `assert_struct!` statements, eliminating:
- Multiple `assert_eq!` calls per test
- Manual `match` statements for variant checking
- Separate assertions for collection properties
- Complex nested matching logic

## Example: Before and After

### Before (Current AST Test Pattern)
```rust
#[test]
fn test_parse_function_with_parameters() {
    let func = parse_function("fn add(x: i32, y: i32) {}");
    
    assert_eq!(func.sig.ident.name, "add");
    assert_eq!(func.sig.inputs.len(), 2);
    
    match &func.sig.inputs[0] {
        FnArg::Typed(pat_type) => {
            match &pat_type.pat {
                Pat::Ident(ident) => {
                    assert_eq!(ident.ident.name, "x");
                }
                _ => panic!("Expected identifier pattern"),
            }
            match &pat_type.ty {
                Type::Path(path_type) => {
                    assert_eq!(path_type.path.segments[0].ident.name, "i32");
                }
                _ => panic!("Expected path type"),
            }
        }
        _ => panic!("Expected typed argument"),
    }
    // Similar for second parameter...
}
```

### After (With Proposed Features)
```rust
#[test]
fn test_parse_function_with_parameters() {
    let func = parse_function("fn add(x: i32, y: i32) {}");
    
    assert_struct!(func, ItemFn {
        sig: Signature {
            ident.name: "add",
            inputs.len(): 2,
            inputs[0]: FnArg::Typed {
                pat: Pat::Ident { ident.name: "x", .. },
                ty: Type::Path { 
                    path.segments[0].ident.name: "i32",
                    ..
                },
            },
            inputs[1]: FnArg::Typed {
                pat: Pat::Ident { ident.name: "y", .. },
                ty: Type::Path {
                    path.segments[0].ident.name: "i32",
                    ..
                },
            },
            ..
        },
        ..
    });
}
```

This roadmap will evolve based on user feedback and real-world usage patterns.