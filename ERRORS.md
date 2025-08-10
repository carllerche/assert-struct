# Improved Errors

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

string pattern mismatch:
  --> `user.name`
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

string pattern mismatch:
  --> `user.name`
   |     name: == name,
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
   |     age: >= 18,
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
  --> `status`
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

comparison mismatch:
  --> `scores[2]`
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

range mismatch:
  --> `person.age`
   |     age: 18..=65,
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

comparison mismatch:
  --> `config.timeout`
   |     timeout: Some(>= 10),
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

comparison mismatch:
  --> `data[15]`
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
   |     postal_code: "90210",
   |                  ^^^^^^^ actual: "98101"
   |     ..
   | } ... }
```

### Example 10: Slice with pattern comparisons (multiple failures)

Code:
```rust
struct Metrics {
    cpu_usage: Vec<u32>,
    memory_mb: u32,
    disk_io: u32,
}

struct Performance {
    baseline: Metrics,
    current: Metrics,
    peak: Metrics,
}

struct System {
    name: String,
    version: String,
    performance: Performance,
}

let system = System {
    name: "prod-server".to_string(),
    version: "2.1.0".to_string(),
    performance: Performance {
        baseline: Metrics {
            cpu_usage: vec![10, 20, 15, 25, 30],
            memory_mb: 512,
            disk_io: 100,
        },
        current: Metrics {
            cpu_usage: vec![
                45, 50, 48, 52, 95, 55, 58, 60, 62, 65,
                68, 70, 72, 98, 78, 80, 82, 85, 88, 90,
                92, 94, 96, 99, 100, 45, 50, 48, 52, 55
            ],
            memory_mb: 1024,
            disk_io: 200,
        },
        peak: Metrics {
            cpu_usage: vec![90, 92, 95, 98, 100],
            memory_mb: 2048,
            disk_io: 500,
        },
    },
};

assert_struct!(system, System {
    performance: Performance {
        current: Metrics {
            cpu_usage: [
                < 90, < 90, < 90, < 90, < 90, < 90, < 90, < 90, < 90, < 90,
                < 90, < 90, < 90, < 90, < 90, < 90, < 90, < 90, < 90, < 90,
                < 90, < 90, < 90, < 90, < 90, < 90, < 90, < 90, < 90, < 90
            ],
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
   | System { ... Metrics {

4 comparison mismatches:
  --> `system.performance.current.cpu_usage`

   | [..., < 90, ...]  // [4]
   |       ^^^^^ actual: 95
   |             failed: 95 < 90

   | [..., < 90, ...]  // [13]
   |       ^^^^^ actual: 98
   |             failed: 98 < 90

   | [..., < 90, ...]  // [23]
   |       ^^^^^ actual: 99
   |             failed: 99 < 90

   | [..., < 90, ...]  // [24]
   |       ^^^^^ actual: 100
   |             failed: 100 < 90

   |     ..
   | } ... }

   = note: 4 of 30 elements failed to match
```

**Why this format:** When slice elements use pattern comparisons (`< 90`, `>= 100`, etc.), we show
each failure individually because the user needs to see why each comparison failed. The "failed: 95 < 90"
line is crucial for understanding the mismatch.

### Example 11: Slice with literal values (diff style)

Code:
```rust
struct Config {
    name: String,
    thresholds: Vec<u32>,
    enabled: bool,
}

struct Settings {
    defaults: Config,
    overrides: Config,
}

struct Application {
    version: String,
    settings: Settings,
}

let app = Application {
    version: "1.0.0".to_string(),
    settings: Settings {
        defaults: Config {
            name: "default".to_string(),
            thresholds: vec![10, 20, 30, 40, 50],
            enabled: true,
        },
        overrides: Config {
            name: "custom".to_string(),
            thresholds: vec![
                10, 20, 30, 45, 50, 60, 70, 80, 90, 100,
                110, 120, 130, 140, 155, 160, 170, 180, 190, 200
            ],
            enabled: false,
        },
    },
};

assert_struct!(app, Application {
    settings: Settings {
        overrides: Config {
            thresholds: [
                10, 20, 30, 40, 50, 60, 70, 80, 90, 100,
                110, 120, 130, 140, 150, 160, 170, 180, 190, 200
            ],
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
   | Application { ... Config {

slice mismatch:
  --> `app.settings.overrides.thresholds`

  Diff < expected / actual >:
    | [
    |     ...
  2 |     30,
  3 |-    40,
    |+    45,
  4 |     50,
    |     ...
 13 |     140,
 14 |-    150,
    |+    155,
 15 |     160,
    |     ...
    | ]

   |     ..
   | } ... }
```

**Why diff format:** When slice elements are literal values or equality comparisons (`[10, 20, 30]` or `== expected_slice`),
we use a unified diff because the user just needs to see which values differ. The diff format is more
compact and easier to scan for differences in concrete values. We show only a context window (1 element
before/after changes) to keep the output focused, using `...` to indicate elided matching elements.



### Example 12: Regex pattern mismatch

Code:
```rust
struct ValidationResult {
    email: String,
    phone: String,
    postal_code: String,
}

let result = ValidationResult {
    email: "invalid.email@".to_string(),
    phone: "+1-555-CALL-ME".to_string(),
    postal_code: "9021A".to_string(),
};

assert_struct!(result, ValidationResult {
    email: =~ r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$",
    phone: =~ r"^\+?[0-9]{1,3}-?[0-9]{3}-?[0-9]{4}$",
    postal_code: =~ r"^\d{5}(-\d{4})?$",
});
```

Target error:
```
assert_struct! failed:
   | ValidationResult {

regex pattern mismatch:
  --> `result.email`
3  |     email: =~ r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$",
   |            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |            actual: "invalid.email@"
   |            pattern failed to match
   |
   = note: regex pattern: ^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$
   |     ..
   | }
```


### Example 13: Tuple pattern mismatch

Code:
```rust
let coordinates = (45.5231, -122.6765, 100.0);  // lat, lon, altitude

assert_struct!(coordinates, (< 40.0, > -120.0, 50.0..=200.0));
```

Target error:
```
assert_struct! failed:
   | (

comparison mismatch:
  --> `coordinates.0`
3  |     < 40.0,
   |     ^^^^^^ actual: 45.5231
   |            failed: 45.5231 < 40.0
   |     > -120.0,
   |     50.0..=200.0
   | )
```

Example with literal values:
```rust
let point = (10, 20, 30);

assert_struct!(point, (10, 25, 30));
```

Target error:
```
assert_struct! failed:
   | (
   |     10,

value mismatch:
  --> `point.1`
3  |     25,
   |     ^^ actual: 20
   |     30
   | )
```

### More examples to add:

#### Pattern-specific examples:
- TODO: Complex nested Option/Result combinations - e.g., `Option<Result<T, E>>` patterns
- TODO: Like trait custom matcher failure - When using `=~ custom_matcher` with a variable/expression

#### Error handling strategies:
- TODO: Multiple field failures - When multiple struct fields fail (not in a slice). Options:
  - Show all failures at once?
  - Fail fast on the first one?
  - Group them somehow?

#### Edge cases and formatting:
- TODO: Very long field paths - What if path is extremely long like `app.config.database.connection_pool.settings.timeout.retry_policy.max_attempts`?
  - Truncate with ellipsis?
  - Wrap to multiple lines?
  - Show abbreviated path?

- TODO: Line number consistency - We show line numbers for literals but not for expressions/patterns. Should we be consistent?

- TODO: Multiple errors vs fail-fast trade-off - We show multiple failures for slices, but what about struct fields? Performance vs completeness considerations.

## Implementation Strategy

### Phase 1: Core Infrastructure
1. **Field path tracking** - Build full paths during macro expansion (e.g., `user.profile.age`)
2. **Pattern AST representation** - Create structured representation of patterns for pretty-printing
3. **Error context collection** - Capture pattern source, field paths, and values at failure point
4. **Span preservation** - Maintain original token spans through macro expansion for line numbers

### Phase 2: Error Message Generation
5. **Custom panic formatter** - Replace basic `panic!()` with rich formatted messages
6. **Pattern renderer** - Pretty-print patterns with consistent indentation and ellipsis for nesting
7. **Diff engine for slices** - Implement unified diff for literal value comparisons
8. **Context window logic** - Show only relevant portions of large patterns/slices

### Phase 3: Advanced Features
9. **Multiple failure collection** (optional) - Gather all failures before panicking for comprehensive errors
10. **Smart truncation** - Handle very long paths and values gracefully
11. **Error categorization** - Different formats for different pattern types (comparison, range, regex, etc.)

### Technical Approach
- Use `#[track_caller]` with helper functions to capture source locations
- Preserve spans: `let location = #original_span capture_location();`
- Generate match expressions that preserve pattern structure in error messages
- Build error messages incrementally with proper formatting and alignment