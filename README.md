# assert-struct

[![CI](https://github.com/carllerche/assert-struct/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/carllerche/assert-struct/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/assert-struct.svg)](https://crates.io/crates/assert-struct)
[![Documentation](https://docs.rs/assert-struct/badge.svg)](https://docs.rs/assert-struct)

Ergonomic structural assertions for Rust tests with helpful error messages.

## What is assert-struct?

`assert-struct` is a procedural macro that enables clean, readable assertions for complex data structures. Instead of verbose field-by-field comparisons, you can assert on nested structures with clear, maintainable syntax. When assertions fail, it provides precise error messages showing exactly what went wrong and where.

## Why use assert-struct?

**The Problem**: Testing complex data structures in Rust is tedious and verbose:

```rust
// Verbose and hard to maintain
assert_eq!(response.user.profile.age, 25);
assert!(response.user.profile.verified);
assert_eq!(response.status.code, 200);
```

**The Solution**: Clean, structural assertions:

```rust
assert_struct!(response, Response {
    user: User {
        profile: Profile {
            age: 25,
            verified: true,
            ..
        },
        ..
    },
    status: Status { code: 200 },
});
```

## When to use assert-struct?

- **API Response Testing** - Validate JSON deserialization results
- **Database Query Testing** - Check returned records match expectations
- **Complex State Validation** - Assert on deeply nested application state
- **Partial Data Matching** - Focus on relevant fields, ignore the rest

## Key Features

- **Partial matching** with `..` - check only the fields you care about
- **Deep nesting** - assert on nested structs without field access chains
- **Map patterns** - `#{ "key": pattern }` for HashMap, BTreeMap, and custom map types
- **Advanced matchers** - comparisons (`> 18`), ranges (`0..100`), regex (`=~ r"pattern"`)
- **Method calls** - `field.len(): 5`, `field.is_some(): true`
- **Collections** - element-wise patterns for `Vec` fields
- **Enums & tuples** - full support for `Option`, `Result`, and custom types
- **Smart pointers** - dereference `Box<T>`, `Rc<T>`, `Arc<T>` with `*field`
- **Helpful errors** - precise error messages with field paths and context

## Quick Start

Add to your `Cargo.toml`:

```toml
[dev-dependencies]
assert-struct = "0.2"
```

Basic usage:

```rust
use assert_struct::assert_struct;

#[derive(Debug)]
struct User {
    name: String,
    age: u32,
    active: bool,
}

let user = User {
    name: "Alice".to_string(),
    age: 30,
    active: true,
};

// Exact match
assert_struct!(user, User {
    name: "Alice",
    age: 30,
    active: true,
});

// Partial match with comparisons
assert_struct!(user, User {
    name: "Alice",
    age: >= 18,  // Adult check
    ..           // Ignore other fields
});
```

## Examples

Common patterns:

```rust
// Method calls
assert_struct!(data, Data {
    items.len(): > 0,
    text.contains("hello"): true,
    ..
});

// Nested field access
assert_struct!(company, Company {
    info.name: "TechCorp",
    info.address.city: "San Francisco",
    info.address.zip: > 90000,
    ..
});

// Collections
assert_struct!(response, Response {
    scores: [> 80.0, >= 90.0, < 100.0],
    ..
});

// Map patterns (HashMap, BTreeMap, custom maps)
assert_struct!(api_response, ApiResponse {
    headers: #{
        "content-type": "application/json",
        "status": == 200,
        ..  // Ignore other headers
    },
    metadata: #{},  // Exactly empty map
    ..
});

// Enums and Options
assert_struct!(result, Result {
    user_data: Some(User {
        age: >= 18,
        verified: true,
        ..
    }),
    ..
});
```

## Documentation

- **[API Documentation](https://docs.rs/assert-struct)** - Complete API reference with examples
- **[Examples Directory](examples/)** - Real-world usage examples
- **Getting Started Guide** - See the main crate documentation

## License

This project is licensed under the MIT License - see the LICENSE file for details.