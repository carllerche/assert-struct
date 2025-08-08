# assert-struct

A Rust procedural macro for ergonomic structural assertions in tests. Write clean, readable assertions for complex data structures without verbose field-by-field comparisons.

## Features

- 🎯 **Partial matching** - Check only the fields you care about with `..`
- 🔍 **Deep nesting** - Assert on nested structs without manual field access chains
- 📝 **Clean syntax** - Rust-like syntax that feels natural
- 🎨 **String literals** - Use `"text"` directly without `.to_string()`
- 📦 **Collection support** - Compare `Vec` with slice literals `[1, 2, 3]`
- 🔢 **Tuple support** - Destructure and compare tuples element by element
- 🔤 **Regex matching** - Use `=~ r"pattern"` for pattern matching on string fields
- 📊 **Comparison operators** - Use `<`, `<=`, `>`, `>=` for numeric comparisons

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

Note: Regex support is enabled by default but can be disabled by turning off default features.

### Comparison Operators

Use comparison operators for numeric field assertions:

```rust
#[derive(Debug)]
struct Person {
    name: String,
    age: u32,
    score: i32,
}

let person = Person {
    name: "Alice".to_string(),
    age: 25,
    score: 85,
};

assert_struct!(person, Person {
    name: "Alice",
    age: >= 18,        // At least 18
    score: > 80,       // Greater than 80
});

// All comparison operators are supported
assert_struct!(person, Person {
    age: < 30,         // Less than 30
    score: <= 100,     // At most 100
    ..
});
```

## How It Works

The macro expands to efficient code using destructuring and `assert_eq!`:

```rust
// This macro invocation:
assert_struct!(user, User {
    name: "Alice",
    age: 30,
});

// Expands to:
{
    let User { ref name, ref age } = user;
    assert_eq!(name, &"Alice");
    assert_eq!(age, &30);
}
```

For nested structures, it generates recursive `assert_struct!` calls, maintaining clean error messages and efficient comparisons.

## Error Messages

When assertions fail, you get clear error messages showing exactly which field didn't match:

```
thread 'test_name' panicked at 'assertion failed: (left == right)
  left: "Bob",
 right: "Alice"'
```

## Development

### Running Tests

```bash
cargo test
```

### Building

```bash
cargo build
```

### Documentation

```bash
cargo doc --open
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.