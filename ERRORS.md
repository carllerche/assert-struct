## Status Quo

Example:
```rust
struct User {
    name: String,
    age: u32,
}

let user = User {
    name: "Alice".to_string(),
    age: 18
};

assert_struct!(user, User {
    name: "Bob",
    ..
});
```

Current error:
```
thread 'test_name_mismatch' panicked at assert-struct/tests/error_test.rs:17:5:
assertion `left == right` failed
  left: "Alice"
 right: "Bob"
```

**Problems:**
- Generic `assertion 'left == right' failed` doesn't indicate which field failed
- Values shown without context about which field (`user.name`)
- No structural context about where in the data structure the mismatch occurred
- For large patterns spanning multiple lines, no indication of exact failure location

## Error Message Target

### Example 1: Basic field mismatch

Code:
```rust
struct User {
    name: String,
    age: u32,
}

let user = User {
    name: "Alice".to_string(),
    age: 18
};

assert_struct!(user, User {
    name: "Bob",
    ..
});
```

Target error:
```
assert_struct! failed:
   | User {

string pattern mismatch at `user.name`:
   |     name: "Bob",
   |           ^^^^^ actual: "Alice"
   |     ..
   | }
```

### Example 2: expression pattern

Code:
```rust
struct User {
    name: String,
    age: u32,
}

let user = User {
    name: "Alice".to_string(),
    age: 18
};

let name = "Bob";

assert_struct!(user, User {
    name: == name,
    ..
});
```

Target error:
```
assert_struct! failed:
   | User {

string pattern mismatch at `user.name`:
3  |     name: == name,
   |           ^^^^^ actual: "Alice"
   |               expected: "Bob"
   |     ..
   | }
```

This error message **includes** the `expected` name value because the pattern is
an expression, so the user does not know what the expected value is based on the
source.

### Example 3: Nested field mismatch

Code:
```rust
struct Profile {
    age: u32,
    verified: bool,
}

struct User {
    name: String,
    profile: Profile,
}

let user = User {
    name: "Alice".to_string(),
    profile: Profile {
        age: 17,
        verified: true,
    }
};

assert_struct!(user, User {
    profile: Profile {
        age: >= 18,
        ..
    },
    ..
});
```

Target error:
```
assert_struct! failed:
   | User { ... Profile {

comparison mismatch:
  --> `user.profile.age`
3  |     age: >= 18,
   |          ^^^^^ actual: 17
   |                failed: 17 >= 18
   |     ..
   | } ... }
```

This error does *not* include the expected value because the pattern is a number
literal. The expected pattern is visible already.

### Example 4: Enum variant mismatch

Code:
```rust
enum Status {
    Active { since: u32 },
    Inactive,
}

let status = Status::Inactive;

assert_struct!(status, Status::Active { since: > 0 });
```

Target error:
```
assert_struct! failed:

enum variant mismatch:
   | Status::Active { since: > 0 }
   | ^^^^^^^^^^^^^^ actual: Status::Inactive
```

### Example 5: Slice pattern with one failure

Code:
```rust
let scores = vec![85, 92, 78, 88];

assert_struct!(scores, [>= 80, >= 80, >= 80, >= 80]);
```

Target error:
```
assert_struct! failed:

comparison mismatch at `scores[2]`:
   | [>= 80, >= 80, >= 80, >= 80]
   |         ^^^^^
   |         actual: 78
   |         failed: 78 >= 80
```

### Example 6: Range pattern failure

Code:
```rust
struct Person {
    name: String,
    age: u32,
}

let person = Person {
    name: "Bob".to_string(),
    age: 75,
};

assert_struct!(person, Person {
    age: 18..=65,
    ..
});
```

Target error:
```
assert_struct! failed:
   | Person {

range mismatch at `person.age`:
3  |     age: 18..=65,
   |          ^^^^^^^^ actual: 75
   |     ..
   | }
```

### Example 7: Option with comparison

Code:
```rust
struct Config {
    timeout: Option<u32>,
}

let config = Config {
    timeout: Some(5),
};

assert_struct!(config, Config {
    timeout: Some(>= 10),
});
```

Target error:
```
assert_struct! failed:
   | Config {

comparison mismatch at `config.timeout`:
3  |     timeout: Some(>= 10),
   |              ^^^^^^^^^^^^ actual: Some(5)
   | }
```

The expected value is **not** included because the pattern is a literal.

### Example 8: Large slice pattern

Code:
```rust
let data = vec![
    10, 20, 30, 40, 50, 60, 70, 80, 90, 100,
    110, 120, 130, 140, 150, 160, 170, 180, 190, 200
];

assert_struct!(data, [
    > 0, > 0, > 0, > 0, > 0, > 0, > 0, > 0, > 0, > 0,
    > 0, > 0, > 0, > 0, > 0, > 200, > 0, > 0, > 0, > 0
]);
```

Target error:
```
assert_struct! failed:

comparison mismatch at `data[15]`:
   | [..., > 200, ...]
   |       ^^^^^^ actual: 160
   |              failed: > 200
   |
   = note: failed at index 15 of 20 elements
```

### Example 9: Deeply nested field mismatch

Code:
```rust
struct Address {
    street: String,
    city: String,
    postal_code: String,
    country: String,
}

struct Contact {
    email: String,
    phone: String,
    address: Address,
    preferred: bool,
}

struct Profile {
    bio: String,
    contact: Contact,
    created_at: u64,
    verified: bool,
    score: u32,
}

struct User {
    id: u64,
    username: String,
    profile: Profile,
    is_active: bool,
}

let user = User {
    id: 12345,
    username: "alice".to_string(),
    profile: Profile {
        bio: "Software engineer".to_string(),
        contact: Contact {
            email: "alice@example.com".to_string(),
            phone: "+1-555-0100".to_string(),
            address: Address {
                street: "123 Main St".to_string(),
                city: "Seattle".to_string(),
                postal_code: "98101".to_string(),
                country: "USA".to_string(),
            },
            preferred: true,
        },
        created_at: 1699564800,
        verified: true,
        score: 95,
    },
    is_active: true,
};

assert_struct!(user, User {
    profile: Profile {
        contact: Contact {
            address: Address {
                postal_code: "90210",
                ..
            },
            ..
        },
        ..
    },
    ..
});
```

Target error:
```
assert_struct! failed:
   | User { ... Address {

string pattern mismatch:
  --> `user.profile.contact.address.postal_code`
5  |     postal_code: "90210",
   |                  ^^^^^^^ actual: "98101"
   |     ..
   | } ... }
```

### More examples to add:
- TODO: Regex pattern mismatch
- TODO: Multiple field failures
- TODO: Tuple pattern mismatch
- TODO: Complex nested Option/Result combinations
- TODO: Like trait custom matcher failure

## Feasibility Analysis

### Feasibility: HIGH
- ✅ Can show file:line:column via `#[track_caller]` and `std::panic::Location::caller()`
- ✅ Can build pattern representation from AST during macro expansion
- ✅ Can track field paths and show exactly which field failed
- ✅ Can annotate the rendered pattern to show failure location
- ✅ Pattern formatting is fully controlled by macro (like `Debug` pretty-printing)
- ✅ **Can get line numbers of pattern fragments!** By setting the span of generated code to match the original pattern token spans, then calling a `#[track_caller]` helper function like `capture_location()`, we can get the exact source location of each pattern element. Example:
  ```rust
  // In macro expansion, preserve the span from the pattern:
  let location = #field_name_token_with_original_span capture_location();

  // Helper function:
  #[track_caller]
  fn capture_location() -> &'static Location<'static> {
      Location::caller()
  }
  ```
  This would report the line/column of the original `name: "Bob"` in the pattern!

### Implementation Strategy
1. Add field path tracking during macro expansion
2. Preserve original token spans when generating assertion code
3. Use `#[track_caller]` helper functions to capture exact pattern locations
4. Build AST-based pattern representation for display
5. Generate comprehensive panic messages with full context and precise location info

## Inspiration

Examples of excellent rustc error messages that we can draw inspiration from:

### Pattern matching errors
```
error[E0308]: mismatched types
  --> src/main.rs:4:9
   |
3  |     match x {
   |           - this expression has type `Option<i32>`
4  |         Some("foo") => {},
   |         ^^^^^^^^^^^ expected `Option<i32>`, found `Option<&str>`
   |
   = note: expected enum `Option<i32>`
              found enum `Option<&'static str>`
```

### Struct field errors
```
error[E0560]: struct `User` has no field named `emial`
  --> src/main.rs:8:9
   |
8  |         emial: "alice@example.com",
   |         ^^^^^ help: a field with a similar name exists: `email`
```

### Type mismatch with context
```
error[E0308]: mismatched types
  --> src/main.rs:12:18
   |
11 |     let age: u32 = user.age;
   |              ---   ^^^^^^^^ expected `u32`, found `String`
   |              |
   |              expected due to this
```

### Multi-line span with annotations
```
error[E0382]: use of moved value: `data`
  --> src/main.rs:6:20
   |
3  |     let data = vec![1, 2, 3];
   |         ---- move occurs because `data` has type `Vec<i32>`
4  |     consume(data);
   |             ---- value moved here
5  |
6  |     println!("{}", data.len());
   |                    ^^^^ value used here after move
```

### Comparison errors
```
error[E0308]: mismatched types
  --> src/main.rs:5:8
   |
5  |     if age > "18" {
   |        --- ^ ---- &str
   |        |   |
   |        |   expected `&str`, found integer
   |        expected because this is `&str`
```

### Assert-like errors (from assert_eq!)
```
thread 'main' panicked at 'assertion failed: `(left == right)`
  left: `Config { port: 8080, host: "localhost" }`,
 right: `Config { port: 3000, host: "localhost" }`', src/main.rs:15:5
```

### Multiple errors in one output
```
error[E0308]: mismatched types
  --> src/main.rs:4:18
   |
4  |     let x: u32 = "hello";
   |            ---   ^^^^^^^ expected `u32`, found `&str`
   |            |
   |            expected due to this

error[E0308]: mismatched types
  --> src/main.rs:5:18
   |
5  |     let y: bool = 42;
   |            ----   ^^ expected `bool`, found integer
   |            |
   |            expected due to this

error[E0384]: cannot assign twice to immutable variable `x`
  --> src/main.rs:7:5
   |
4  |     let x: u32 = "hello";
   |         - first assignment to `x`
...
7  |     x = 10;
   |     ^^^^^^ cannot assign twice to immutable variable

error: aborting due to 3 previous errors
```

### Key patterns to emulate:
- **Clear hierarchy**: Main error → file location → detailed explanation
- **Visual indicators**: Arrows, underlines, and tree-like connectors
- **Contextual information**: Show surrounding context when helpful
- **Expected vs Found**: Consistent terminology throughout
- **Notes and help**: Additional context in `= note:` and `= help:` sections
- **Precise locations**: Exact line:column for the problematic code
- **Type information**: Show full types when relevant for understanding