# assert-struct

[![CI](https://github.com/carllerche/assert-struct/workflows/CI/badge.svg)](https://github.com/carllerche/assert-struct/actions)
[![Crates.io](https://img.shields.io/crates/v/assert-struct.svg)](https://crates.io/crates/assert-struct)
[![Documentation](https://docs.rs/assert-struct/badge.svg)](https://docs.rs/assert-struct)

Ergonomic structural assertions for Rust tests. `assert-struct` is a procedural
macro that enables clean, readable assertions for complex data structures
without verbose field-by-field comparisons. It's the testing tool you need when
`assert_eq!` isn't enough and manually comparing fields is too cumbersome.

## Quick Example

```rust
use assert_struct::assert_struct;

#[derive(Debug)]
struct User {
    name: String,
    age: u32,
    email: String,
    role: String,
}

let user = User {
    name: "Alice".to_string(),
    age: 30,
    email: "alice@example.com".to_string(),
    role: "admin".to_string(),
};

// Only check the fields you care about
assert_struct!(user, User {
    name: "Alice",
    age: 30,
    ..  // Ignore email and role
});
```

## Why assert-struct?

Testing complex data structures in Rust often involves tedious boilerplate:

```rust
// Without assert-struct: verbose and hard to read
assert_eq!(response.user.profile.age, 25);
assert!(response.user.profile.verified);
assert_eq!(response.status.code, 200);

// With assert-struct: clean and intuitive
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

## Features

### Core Capabilities

- **Partial matching** - Use `..` to check only the fields you care about
- **Deep nesting** - Assert on nested structs without manual field access chains
- **String literals** - Compare `String` fields directly with `"text"` literals
- **Collections** - Assert on `Vec` fields with element-wise patterns
- **Tuples** - Full support for multi-field tuples with advanced patterns
- **Enum support** - Match on `Option`, `Result`, and custom enum variants

### Advanced Matchers

- **Comparison operators** - Use `<`, `<=`, `>`, `>=` for numeric field assertions
- **Equality operators** - Use `==` and `!=` for explicit equality/inequality checks
- **Range patterns** - Use `18..=65`, `0.0..100.0` for range matching
- **Regex patterns** - Match string fields with regular expressions using `=~ r"pattern"`
- **Slice patterns** - Element-wise patterns for `Vec` fields like `[> 0, < 10, == 5]`
- **Advanced enum patterns** - Use comparison operators and regex inside `Some()` and other variants

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
assert-struct = "0.1.0"
```

## Usage

### Basic Assertions

```rust
use assert_struct::assert_struct;

#[derive(Debug)]
struct User {
    name: String,
    age: u32,
}

let user = User {
    name: "Alice".to_string(),
    age: 30,
};

// Assert all fields (exhaustive)
assert_struct!(user, User {
    name: "Alice",
    age: 30,
});
```

### Partial Matching

Use `..` to check only specific fields:

```rust
assert_struct!(user, User {
    name: "Alice",
    ..  // Don't check other fields
});
```

### Nested Structures

Assert on deeply nested structs without verbose field access:

```rust
#[derive(Debug)]
struct Address {
    street: String,
    city: String,
    zip: u32,
}

#[derive(Debug)]
struct Person {
    name: String,
    age: u32,
    address: Address,
}

let person = Person {
    name: "Bob".to_string(),
    age: 25,
    address: Address {
        street: "123 Main St".to_string(),
        city: "Springfield".to_string(),
        zip: 12345,
    },
};

assert_struct!(person, Person {
    name: "Bob",
    age: 25,
    address: Address {
        street: "123 Main St",
        city: "Springfield",
        zip: 12345,
    },
});

// Or with partial matching on nested structs
assert_struct!(person, Person {
    name: "Bob",
    address: Address {
        city: "Springfield",
        ..
    },
    ..
});
```

### Collections

Element-wise pattern matching for vectors:

```rust
#[derive(Debug)]
struct Data {
    values: Vec<u32>,
    names: Vec<String>,
}

let data = Data {
    values: vec![5, 15, 25],
    names: vec!["alice".to_string(), "bob".to_string()],
};

// Exact matching
assert_struct!(data, Data {
    values: [5, 15, 25],
    names: ["alice", "bob"],  // String literals work!
});

// Advanced patterns with comparison operators
assert_struct!(data, Data {
    values: [> 0, < 20, == 25],  // Different matcher for each element
    names: ["alice", "bob"],
});
```

### Tuples

Full support for multi-field tuples with advanced pattern matching:

```rust
#[derive(Debug)]
struct Data {
    point: (i32, i32),
    metadata: (String, u32, bool),
    nested: ((f64, f64), (String, bool)),
}

let data = Data {
    point: (15, 25),
    metadata: ("info".to_string(), 100, true),
    nested: ((1.5, 2.5), ("test".to_string(), false)),
};

// Basic tuple matching
assert_struct!(data, Data {
    point: (15, 25),
    metadata: ("info", 100, true),  // String literals work!
    nested: ((1.5, 2.5), ("test", false)),  // Nested tuples!
});

// Advanced patterns with comparisons
assert_struct!(data, Data {
    point: (> 10, < 30),  // Comparison operators in tuples
    metadata: ("info", >= 50, true),
    nested: ((> 1.0, <= 3.0), ("test", false)),
});

// Enum tuple variants with multiple fields
#[derive(Debug, PartialEq)]
enum Event {
    Click(i32, i32),
    Drag(i32, i32, i32, i32),
    Scroll(f64, String),
}

struct Log {
    event: Event,
}

let log = Log {
    event: Event::Drag(10, 20, 110, 120),
};

assert_struct!(log, Log {
    event: Event::Drag(>= 0, >= 0, < 200, < 200),  // Comparisons in enum tuples
});
```

### Regex Patterns

Use `=~ r"pattern"` to match string fields against regular expressions:

```rust
#[derive(Debug)]
struct User {
    username: String,
    email: String,
}

let user = User {
    username: "user_123".to_string(),
    email: "alice@example.com".to_string(),
};

assert_struct!(user, User {
    username: =~ r"^user_\d+$",  // Must start with "user_" followed by digits
    email: =~ r"^[^@]+@[^@]+\.[^@]+$",  // Basic email pattern
});
```

Note: Regex support is enabled by default but can be disabled by turning off
default features.

### Advanced Pattern Matching with Like Trait

The `Like` trait enables flexible pattern matching beyond simple equality. You can use regex patterns from variables, pre-compiled regex, or even implement custom matching logic:

```rust
use assert_struct::{assert_struct, Like};
use regex::Regex;

#[derive(Debug)]
struct User {
    email: String,
    username: String,
}

let user = User {
    email: "alice@example.com".to_string(),
    username: "alice_doe".to_string(),
};

// Use pre-compiled regex for better performance
let email_regex = Regex::new(r"^[^@]+@example\.com$").unwrap();
assert_struct!(user, User {
    email: =~ email_regex,
    ..
});

// Use patterns from variables
let username_pattern = r"^[a-z_]+$";
assert_struct!(user, User {
    username: =~ username_pattern,
    ..
});

// Custom Like implementation
struct DomainPattern {
    domain: String,
}

impl Like<DomainPattern> for String {
    fn like(&self, pattern: &DomainPattern) -> bool {
        self.ends_with(&format!("@{}", pattern.domain))
    }
}

let domain = DomainPattern { domain: "example.com".to_string() };
assert_struct!(user, User {
    email: =~ domain,
    ..
});
```

The Like trait also includes advanced utility types for common patterns:

```rust
use assert_struct::{CaseInsensitive, Prefix, Suffix};

#[derive(Debug)]
struct Data {
    name: String,
    code: String,
    path: String,
}

let data = Data {
    name: "Alice Smith".to_string(),
    code: "PREFIX_123_SUFFIX".to_string(),
    path: "/home/user/file.txt".to_string(),
};

assert_struct!(data, Data {
    name: =~ CaseInsensitive("alice smith".to_string()),  // Case-insensitive match
    code: =~ Prefix("PREFIX".to_string()),                 // Starts with PREFIX
    path: =~ Suffix(".txt".to_string()),                   // Ends with .txt
});

### Option and Result Types

Native support for Rust's standard `Option` and `Result` types:

```rust
#[derive(Debug)]
struct UserProfile {
    name: String,
    age: Option<u32>,
    email_verified: Result<bool, String>,
}

let profile = UserProfile {
    name: "Alice".to_string(),
    age: Some(30),
    email_verified: Ok(true),
};

assert_struct!(profile, UserProfile {
    name: "Alice",
    age: Some(30),
    email_verified: Ok(true),
});

// Advanced patterns with Option
assert_struct!(profile, UserProfile {
    name: "Alice",
    age: Some(>= 18),  // Adult check inside Some
    email_verified: Ok(true),
});
```

### Custom Enums

Full support for custom enum types with all variant types:

```rust
#[derive(Debug, PartialEq)]
enum Status {
    Active,
    Inactive,
    Pending { since: String },
    Error { code: i32, message: String },
}

#[derive(Debug)]
struct Account {
    id: u32,
    status: Status,
}

let account = Account {
    id: 1,
    status: Status::Pending {
        since: "2024-01-01".to_string(),
    },
};

assert_struct!(account, Account {
    id: 1,
    status: Status::Pending {
        since: "2024-01-01",
    },
});

// Partial matching on enum fields
let error_account = Account {
    id: 2,
    status: Status::Error {
        code: 500,
        message: "Internal error".to_string(),
    },
};

assert_struct!(error_account, Account {
    id: 2,
    status: Status::Error {
        code: 500,
        ..  // Ignore the message field
    },
});
```

### Comparison and Equality Operators

Perfect for range checks, threshold validations, and explicit equality tests:

```rust
#[derive(Debug)]
struct Metrics {
    cpu_usage: f64,
    memory_mb: u32,
    response_time_ms: u32,
    status: String,
}

let metrics = Metrics {
    cpu_usage: 75.5,
    memory_mb: 1024,
    response_time_ms: 150,
    status: "ok".to_string(),
};

assert_struct!(metrics, Metrics {
    cpu_usage: < 80.0,         // Less than 80%
    memory_mb: <= 2048,         // At most 2GB
    response_time_ms: < 200,    // Under 200ms
    status: == "ok",            // Exact equality
});

// Inequality checks
assert_struct!(metrics, Metrics {
    status: != "error",         // Not equal to "error"
    memory_mb: != 0,            // Not zero
    ..
});

// Complex expressions
fn get_threshold() -> f64 { 75.0 }

assert_struct!(metrics, Metrics {
    cpu_usage: < get_threshold() + 5.0,  // Function calls and arithmetic
    memory_mb: >= config.min_memory,     // Field access
    response_time_ms: < limits[2],       // Array indexing
    ..
});
```

## Real-World Examples

### Testing API Responses

```rust
#[derive(Debug)]
struct ApiResponse {
    status: String,
    data: UserData,
    timestamp: i64,
}

// After deserializing JSON response
assert_struct!(response, ApiResponse {
    status: "success",
    data: UserData {
        username: "testuser",
        permissions: vec!["read".to_string(), "write".to_string()],
        ..  // Don't check the generated ID
    },
    ..  // Don't check timestamp
});
```

### Testing Database Records

```rust
// After fetching from database
assert_struct!(product, Product {
    name: "Laptop",
    price: > 500.0,      // Price above minimum
    stock: > 0,          // In stock
    category: "Electronics",
    ..  // Ignore auto-generated ID
});
```

### Testing State Changes

```rust
// After game action
assert_struct!(state, GameState {
    score: >= 1000,      // Minimum score achieved
    level: 3,            // Reached level 3
    player: Player {
        health: > 0,     // Still alive
        inventory: vec!["sword".to_string(), "shield".to_string()],  // Has required items
        ..  // Position doesn't matter
    },
});
```

## Crate Features

| Feature | Default | Description |
|---------|---------|-------------|
| `regex` | **Yes** | Enables regex pattern matching with the `=~ r"pattern"` syntax |

To disable regex support (and avoid the regex dependency):

```toml
[dependencies]
assert-struct = { version = "0.1", default-features = false }
```

## Documentation

See the [full documentation](https://docs.rs/assert-struct) for:
- Complete syntax reference
- All supported matchers
- Advanced usage patterns
- Compilation error examples

## Development

```bash
cargo test           # Run all tests
cargo test --doc     # Test documentation examples
cargo doc --open     # View local documentation
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.