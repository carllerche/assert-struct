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
- **Collections** - Assert on `Vec` fields using slice syntax `[1, 2, 3]`
- **Tuples** - Destructure and compare tuple fields element by element
- **Enum support** - Match on `Option`, `Result`, and custom enum variants

### Advanced Matchers

- **Comparison operators** - Use `<`, `<=`, `>`, `>=` for numeric field assertions
- **Regex patterns** - Match string fields with regular expressions using `=~ r"pattern"`
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

Compare vectors with slice syntax:

```rust
#[derive(Debug)]
struct Data {
    values: Vec<u32>,
    name: String,
}

let data = Data {
    values: vec![1, 2, 3],
    name: "test".to_string(),
};

assert_struct!(data, Data {
    values: [1, 2, 3],  // Use slice syntax for Vec
    name: "test",
});
```

### Tuples

Tuples are destructured and compared element by element:

```rust
#[derive(Debug)]
struct Container {
    data: (u32, String),
    id: u32,
}

let container = Container {
    data: (100, "hello".to_string()),
    id: 1,
};

assert_struct!(container, Container {
    data: (100, "hello"),  // String literal works!
    id: 1,
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

### Comparison Operators

Perfect for range checks and threshold validations:

```rust
#[derive(Debug)]
struct Metrics {
    cpu_usage: f64,
    memory_mb: u32,
    response_time_ms: u32,
}

let metrics = Metrics {
    cpu_usage: 75.5,
    memory_mb: 1024,
    response_time_ms: 150,
};

assert_struct!(metrics, Metrics {
    cpu_usage: < 80.0,        // Less than 80%
    memory_mb: <= 2048,        // At most 2GB
    response_time_ms: < 200,   // Under 200ms
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
        permissions: ["read", "write"],
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
        inventory: ["sword", "shield"],  // Has required items
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