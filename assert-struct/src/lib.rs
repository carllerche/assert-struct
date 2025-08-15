//! Ergonomic structural assertions for Rust tests with helpful error messages.
//!
//! `assert-struct` is a procedural macro that enables clean, readable assertions for complex
//! data structures without verbose field-by-field comparisons. When assertions fail, it provides
//! clear, actionable error messages showing exactly what went wrong, including field paths and
//! expected vs actual values. It's the testing tool you need when `assert_eq!` isn't enough
//! and manually comparing fields is too cumbersome.
//!
//! # Quick Example
//!
//! ```rust
//! use assert_struct::assert_struct;
//!
//! #[derive(Debug)]
//! struct User {
//!     name: String,
//!     age: u32,
//!     email: String,
//!     role: String,
//! }
//!
//! let user = User {
//!     name: "Alice".to_string(),
//!     age: 30,
//!     email: "alice@example.com".to_string(),
//!     role: "admin".to_string(),
//! };
//!
//! // Only check the fields you care about
//! assert_struct!(user, User {
//!     name: "Alice",
//!     age: 30,
//!     ..  // Ignore email and role
//! });
//! ```
//!
//! # Why assert-struct?
//!
//! Testing complex data structures in Rust often involves tedious boilerplate:
//!
//! ```rust
//! # struct Response { user: User, status: Status }
//! # struct User { id: String, profile: Profile }
//! # struct Profile { age: u32, verified: bool }
//! # struct Status { code: i32 }
//! # let response = Response {
//! #     user: User {
//! #         id: "123".to_string(),
//! #         profile: Profile { age: 25, verified: true }
//! #     },
//! #     status: Status { code: 200 }
//! # };
//! // Without assert-struct: verbose and hard to read
//! assert_eq!(response.user.profile.age, 25);
//! assert!(response.user.profile.verified);
//! assert_eq!(response.status.code, 200);
//! ```
//!
//! With `assert-struct`, the same test becomes clear and maintainable:
//!
//! ```rust
//! # use assert_struct::assert_struct;
//! # #[derive(Debug)]
//! # struct Response { user: User, status: Status }
//! # #[derive(Debug)]
//! # struct User { id: String, profile: Profile }
//! # #[derive(Debug)]
//! # struct Profile { age: u32, verified: bool }
//! # #[derive(Debug)]
//! # struct Status { code: i32 }
//! # let response = Response {
//! #     user: User {
//! #         id: "123".to_string(),
//! #         profile: Profile { age: 25, verified: true }
//! #     },
//! #     status: Status { code: 200 }
//! # };
//! // With assert-struct: clean and intuitive
//! assert_struct!(response, Response {
//!     user: User {
//!         profile: Profile {
//!             age: 25,
//!             verified: true,
//!             ..
//!         },
//!         ..
//!     },
//!     status: Status { code: 200 },
//! });
//! ```
//!
//! # Overview
//!
//! `assert-struct` provides a single macro that transforms structural patterns into assertions.
//! It excels at testing:
//!
//! - **API responses** - Verify JSON deserialization results
//! - **Database queries** - Check returned records match expectations
//! - **Complex state** - Assert on deeply nested application state
//! - **Partial data** - Focus on relevant fields, ignore the rest
//!
//! The macro uses Rust's pattern matching syntax, making it feel natural and familiar. It generates
//! efficient code that provides clear error messages when assertions fail.
//!
//! # Features
//!
//! ## Core Capabilities
//!
//! - **Helpful Error Messages** - Clear, actionable errors showing field paths, expected vs actual values
//! - **Partial Matching** - Use `..` to check only the fields you care about
//! - **Deep Nesting** - Assert on nested structs without manual field access chains
//! - **String Literals** - Compare `String` fields directly with `"text"` literals
//! - **Collections** - Assert on `Vec` fields with element-wise patterns `[> 0, < 10, == 5]`
//! - **Tuples** - Full support for multi-field tuples with advanced patterns
//! - **Enum Support** - Match on `Option`, `Result`, and custom enum variants
//!
//! ## Advanced Matchers
//!
//! - **Comparison Operators** - Use `<`, `<=`, `>`, `>=` for numeric field assertions
//! - **Equality Operators** - Use `==` and `!=` for explicit equality/inequality checks
//! - **Range Patterns** - Use `18..=65`, `0.0..100.0`, `0..` for range matching
//! - **Regex Patterns** - Match string fields with regular expressions using `=~ r"pattern"`
//! - **Advanced Enum Patterns** - Use comparison operators, ranges, and regex inside `Some()` and other variants
//!
//! # Helpful Error Messages
//!
//! When assertions fail, `assert-struct` provides detailed error messages that make debugging easy:
//!
//! ```rust,should_panic
//! # use assert_struct::assert_struct;
//! # #[derive(Debug)]
//! # struct User { name: String, age: u32 }
//! # let user = User { name: "Alice".to_string(), age: 25 };
//! assert_struct!(user, User {
//!     name: "Bob",  // This will fail
//!     age: 25,
//! });
//! // Error output:
//! // assert_struct! failed:
//! //
//! // value mismatch:
//! //   --> `user.name` (src/lib.rs:120)
//! //   actual: "Alice"
//! //   expected: "Bob"
//! ```
//!
//! For complex patterns, the error shows the exact pattern that failed:
//!
//! ```rust,should_panic
//! # use assert_struct::assert_struct;
//! # #[derive(Debug)]
//! # struct Stats { score: u32, level: u32 }
//! # let stats = Stats { score: 50, level: 3 };
//! assert_struct!(stats, Stats {
//!     score: > 100,  // This will fail
//!     level: 3,
//! });
//! // Error output:
//! // assert_struct! failed:
//! //
//! // comparison mismatch:
//! //   --> `stats.score` (src/lib.rs:135)
//! //   actual: 50
//! //   expected: > 100
//! ```
//!
//! # Usage
//!
//! ## Basic Assertions
//!
//! The simplest use case is asserting all fields of a struct:
//!
//! ```rust
//! # use assert_struct::assert_struct;
//! # #[derive(Debug)]
//! # struct Point { x: i32, y: i32 }
//! let point = Point { x: 10, y: 20 };
//!
//! assert_struct!(point, Point {
//!     x: 10,
//!     y: 20,
//! });
//! ```
//!
//! ## Partial Matching
//!
//! Use `..` to ignore fields you don't want to check:
//!
//! ```rust
//! # use assert_struct::assert_struct;
//! # #[derive(Debug)]
//! # struct User { id: u64, name: String, email: String, created_at: String }
//! # let user = User {
//! #     id: 1,
//! #     name: "Alice".to_string(),
//! #     email: "alice@example.com".to_string(),
//! #     created_at: "2024-01-01".to_string(),
//! # };
//! // Only verify name and email, ignore id and created_at
//! assert_struct!(user, User {
//!     name: "Alice",
//!     email: "alice@example.com",
//!     ..
//! });
//! ```
//!
//! ## Nested Structures
//!
//! Assert on deeply nested data without repetitive field access:
//!
//! ```rust
//! # use assert_struct::assert_struct;
//! # #[derive(Debug)]
//! # struct Order { id: u64, customer: Customer, items: Vec<String> }
//! # #[derive(Debug)]
//! # struct Customer { name: String, address: Address }
//! # #[derive(Debug)]
//! # struct Address { city: String, country: String }
//! # let order = Order {
//! #     id: 1001,
//! #     customer: Customer {
//! #         name: "Bob".to_string(),
//! #         address: Address {
//! #             city: "Paris".to_string(),
//! #             country: "France".to_string(),
//! #         }
//! #     },
//! #     items: vec!["Book".to_string(), "Pen".to_string()],
//! # };
//! assert_struct!(order, Order {
//!     customer: Customer {
//!         address: Address {
//!             city: "Paris",
//!             country: "France",
//!         },
//!         ..
//!     },
//!     ..
//! });
//! ```
//!
//! ## Option and Result Types
//!
//! Native support for Rust's standard `Option` and `Result` types:
//!
//! ```rust
//! # use assert_struct::assert_struct;
//! # #[derive(Debug)]
//! # struct UserProfile { name: String, age: Option<u32>, verified: Result<bool, String> }
//! # let profile = UserProfile {
//! #     name: "Alice".to_string(),
//! #     age: Some(30),
//! #     verified: Ok(true),
//! # };
//! assert_struct!(profile, UserProfile {
//!     name: "Alice",
//!     age: Some(30),
//!     verified: Ok(true),
//! });
//!
//! // Advanced patterns with Option
//! assert_struct!(profile, UserProfile {
//!     name: "Alice",
//!     age: Some(>= 18),  // Adult check inside Some
//!     verified: Ok(true),
//! });
//! ```
//!
//! ## Custom Enums
//!
//! Full support for custom enum types with all variant types:
//!
//! ```rust
//! # use assert_struct::assert_struct;
//! # #[derive(Debug, PartialEq)]
//! # enum Status { Active, Pending { since: String } }
//! # #[derive(Debug)]
//! # struct Account { id: u32, status: Status }
//! # let account = Account {
//! #     id: 1,
//! #     status: Status::Pending { since: "2024-01-01".to_string() },
//! # };
//! assert_struct!(account, Account {
//!     id: 1,
//!     status: Status::Pending {
//!         since: "2024-01-01",
//!     },
//! });
//! ```
//!
//! ## Slices and Vectors
//!
//! Element-wise pattern matching for `Vec` fields:
//!
//! ```rust
//! # use assert_struct::assert_struct;
//! # #[derive(Debug)]
//! # struct Data {
//! #     values: Vec<i32>,
//! #     names: Vec<String>,
//! # }
//! # let data = Data {
//! #     values: vec![5, 15, 25],
//! #     names: vec!["alice".to_string(), "bob".to_string()],
//! # };
//! // Exact matching
//! assert_struct!(data, Data {
//!     values: [5, 15, 25],
//!     names: ["alice", "bob"],
//! });
//!
//! // Comparison patterns for each element
//! assert_struct!(data, Data {
//!     values: [> 0, < 20, == 25],  // Different matcher for each element
//!     names: ["alice", "bob"],
//! });
//! ```
//!
//! ## Tuples
//!
//! Full support for multi-field tuples with advanced pattern matching:
//!
//! ```rust
//! # use assert_struct::assert_struct;
//! # #[derive(Debug)]
//! # struct Data {
//! #     point: (i32, i32),
//! #     metadata: (String, u32, bool),
//! # }
//! # let data = Data {
//! #     point: (15, 25),
//! #     metadata: ("info".to_string(), 100, true),
//! # };
//! // Basic tuple matching
//! assert_struct!(data, Data {
//!     point: (15, 25),
//!     metadata: ("info", 100, true),  // String literals work!
//! });
//!
//! // Advanced patterns with comparisons
//! assert_struct!(data, Data {
//!     point: (> 10, < 30),  // Comparison operators in tuples
//!     metadata: ("info", >= 50, true),
//! });
//! ```
//!
//! Tuples can also appear in enum variants:
//!
//! ```rust
//! # use assert_struct::assert_struct;
//! # #[derive(Debug, PartialEq)]
//! # enum Event {
//! #     Click(i32, i32),
//! #     Drag(i32, i32, i32, i32),
//! # }
//! # #[derive(Debug)]
//! # struct Log { event: Event }
//! # let log = Log { event: Event::Drag(10, 20, 110, 120) };
//! assert_struct!(log, Log {
//!     event: Event::Drag(>= 0, >= 0, < 200, < 200),  // Comparisons in enum tuples
//! });
//! ```
//!
//! ## Comparison Operators
//!
//! Perfect for range checks and threshold validations:
//!
//! ```rust
//! # use assert_struct::assert_struct;
//! # #[derive(Debug)]
//! # struct Metrics { cpu_usage: f64, memory_mb: u32, response_time_ms: u32 }
//! # let metrics = Metrics { cpu_usage: 75.5, memory_mb: 1024, response_time_ms: 150 };
//! assert_struct!(metrics, Metrics {
//!     cpu_usage: < 80.0,        // Less than 80%
//!     memory_mb: <= 2048,        // At most 2GB
//!     response_time_ms: < 200,   // Under 200ms
//! });
//! ```
//!
//! ## Regex Patterns
//!
//! Validate string formats and patterns:
//!
//! ```rust
//! # #[cfg(feature = "regex")]
//! # {
//! # use assert_struct::assert_struct;
//! # #[derive(Debug)]
//! # struct Account { username: String, user_id: String, email: String }
//! # let account = Account {
//! #     username: "alice_doe".to_string(),
//! #     user_id: "usr_123456".to_string(),
//! #     email: "alice@company.com".to_string(),
//! # };
//! assert_struct!(account, Account {
//!     username: =~ r"^[a-z_]+$",           // Lowercase letters and underscores
//!     user_id: =~ r"^usr_\d{6}$",          // Specific ID format
//!     email: =~ r"@company\.com$",         // Company email domain
//! });
//! # }
//! ```
//!
//! # Examples
//!
//! ## Testing API Responses
//!
//! ```rust
//! # use assert_struct::assert_struct;
//! #[derive(Debug)]
//! struct ApiResponse {
//!     status: String,
//!     data: UserData,
//!     timestamp: i64,
//! }
//!
//! #[derive(Debug)]
//! struct UserData {
//!     id: u64,
//!     username: String,
//!     permissions: Vec<String>,
//! }
//!
//! # let response = ApiResponse {
//! #     status: "success".to_string(),
//! #     data: UserData {
//! #         id: 42,
//! #         username: "testuser".to_string(),
//! #         permissions: vec!["read".to_string(), "write".to_string()],
//! #     },
//! #     timestamp: 1234567890,
//! # };
//! // After deserializing JSON response
//! assert_struct!(response, ApiResponse {
//!     status: "success",
//!     data: UserData {
//!         username: "testuser",
//!         permissions: vec!["read".to_string(), "write".to_string()],
//!         ..  // Don't check the generated ID
//!     },
//!     ..  // Don't check timestamp
//! });
//! ```
//!
//! ## Testing Database Records
//!
//! ```rust
//! # use assert_struct::assert_struct;
//! # #[derive(Debug)]
//! # struct Product {
//! #     id: u64,
//! #     name: String,
//! #     price: f64,
//! #     stock: u32,
//! #     category: String,
//! # }
//! # let product = Product {
//! #     id: 1,
//! #     name: "Laptop".to_string(),
//! #     price: 999.99,
//! #     stock: 15,
//! #     category: "Electronics".to_string(),
//! # };
//! // After fetching from database
//! assert_struct!(product, Product {
//!     name: "Laptop",
//!     price: > 500.0,      // Price above minimum
//!     stock: > 0,          // In stock
//!     category: "Electronics",
//!     ..  // Ignore auto-generated ID
//! });
//! ```
//!
//! ## Testing State Changes
//!
//! ```rust
//! # use assert_struct::assert_struct;
//! # #[derive(Debug)]
//! # struct GameState {
//! #     score: u32,
//! #     level: u32,
//! #     player: Player,
//! # }
//! # #[derive(Debug)]
//! # struct Player {
//! #     health: u32,
//! #     position: (i32, i32),
//! #     inventory: Vec<String>,
//! # }
//! # let state = GameState {
//! #     score: 1500,
//! #     level: 3,
//! #     player: Player {
//! #         health: 75,
//! #         position: (10, 20),
//! #         inventory: vec!["sword".to_string(), "shield".to_string()],
//! #     },
//! # };
//! // After game action
//! assert_struct!(state, GameState {
//!     score: >= 1000,      // Minimum score achieved
//!     level: 3,            // Reached level 3
//!     player: Player {
//!         health: > 0,     // Still alive
//!         inventory: vec!["sword".to_string(), "shield".to_string()],  // Has required items
//!         ..  // Position doesn't matter
//!     },
//! });
//! ```
//!
//! # Crate Features
//!
//! This crate has the following Cargo features:
//!
//! | Feature | Default | Description |
//! |---------|---------|-------------|
//! | `regex` | **Yes** | Enables regex pattern matching with the `=~ r"pattern"` syntax |
//!
//! To disable regex support (and avoid the regex dependency):
//!
//! ```toml
//! [dependencies]
//! assert-struct = { version = "0.1", default-features = false }
//! ```
//!
//! # See Also
//!
//! - [`assert_struct!`] - The main macro reference documentation with complete syntax details
//! - [GitHub Repository](https://github.com/carllerche/assert-struct) - Source code and issue tracking
//! - [Examples](https://github.com/carllerche/assert-struct/tree/main/tests) - More usage examples

// Re-export the procedural macro
pub use assert_struct_macros::assert_struct;

// Error handling module
pub mod error;

// Structured error document module (new architecture)
pub mod error_document;

// New two-pass error rendering system
mod error_v2;

// Hidden module for macro support functions
#[doc(hidden)]
pub mod __macro_support {
    pub use crate::error::{
        ErrorContext, ErrorType, format_errors_with_root, format_errors_with_root_dispatch,
    };
}

/// A trait for pattern matching, similar to `PartialEq` but for flexible matching.
///
/// The `Like` trait enables custom pattern matching logic beyond simple equality.
/// It's primarily used with the `=~` operator in `assert_struct!` macro to support
/// regex patterns, custom matching logic, and other pattern-based comparisons.
///
/// # Examples
///
/// ## Basic String Pattern Matching
///
/// ```
/// # #[cfg(feature = "regex")]
/// # {
/// use assert_struct::Like;
///
/// // Using Like trait directly
/// let text = "hello@example.com";
/// assert!(text.like(&r".*@example\.com"));
/// # }
/// ```
///
/// ## Custom Implementation
///
/// ```
/// use assert_struct::Like;
///
/// struct EmailAddress(String);
///
/// struct DomainPattern {
///     domain: String,
/// }
///
/// impl Like<DomainPattern> for EmailAddress {
///     fn like(&self, pattern: &DomainPattern) -> bool {
///         self.0.ends_with(&format!("@{}", pattern.domain))
///     }
/// }
///
/// let email = EmailAddress("user@example.com".to_string());
/// let pattern = DomainPattern { domain: "example.com".to_string() };
/// assert!(email.like(&pattern));
/// ```
pub trait Like<Rhs = Self> {
    /// Returns `true` if `self` matches the pattern `other`.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "regex")]
    /// # {
    /// use assert_struct::Like;
    ///
    /// let s = "test123";
    /// assert!(s.like(&r"\w+\d+"));
    /// # }
    /// ```
    fn like(&self, other: &Rhs) -> bool;
}

// String/&str implementations for regex pattern matching
#[cfg(feature = "regex")]
mod like_impls {
    use super::Like;

    /// Implementation of Like for String with &str patterns (interpreted as regex)
    impl Like<&str> for String {
        fn like(&self, pattern: &&str) -> bool {
            regex::Regex::new(pattern)
                .map(|re| re.is_match(self))
                .unwrap_or(false)
        }
    }

    /// Implementation of Like for String with String patterns (interpreted as regex)
    impl Like<String> for String {
        fn like(&self, pattern: &String) -> bool {
            self.like(&pattern.as_str())
        }
    }

    /// Implementation of Like for &str with &str patterns (interpreted as regex)
    impl Like<&str> for &str {
        fn like(&self, pattern: &&str) -> bool {
            regex::Regex::new(pattern)
                .map(|re| re.is_match(self))
                .unwrap_or(false)
        }
    }

    /// Implementation of Like for &str with String patterns (interpreted as regex)
    impl Like<String> for &str {
        fn like(&self, pattern: &String) -> bool {
            self.like(&pattern.as_str())
        }
    }

    /// Implementation of Like for String with pre-compiled Regex
    impl Like<regex::Regex> for String {
        fn like(&self, pattern: &regex::Regex) -> bool {
            pattern.is_match(self)
        }
    }

    /// Implementation of Like for &str with pre-compiled Regex
    impl Like<regex::Regex> for &str {
        fn like(&self, pattern: &regex::Regex) -> bool {
            pattern.is_match(self)
        }
    }
}
