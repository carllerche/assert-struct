//! # assert-struct: Ergonomic Structural Assertions
//!
//! `assert-struct` is a procedural macro that enables clean, readable assertions for complex
//! data structures without verbose field-by-field comparisons. When assertions fail, it provides
//! clear, actionable error messages showing exactly what went wrong, including field paths and
//! expected vs actual values.
//!
//! This comprehensive guide teaches you how to use `assert-struct` effectively in your tests.
//! After reading this documentation, you'll be familiar with all capabilities and able to
//! leverage the full power of structural assertions.
//!
//! # Table of Contents
//!
//! - [Quick Start](#quick-start)
//! - [Core Concepts](#core-concepts)
//!   - [Basic Assertions](#basic-assertions)
//!   - [Partial Matching](#partial-matching)
//!   - [Nested Structures](#nested-structures)
//! - [Pattern Types](#pattern-types)
//!   - [Comparison Operators](#comparison-operators)
//!   - [Equality Operators](#equality-operators)
//!   - [Range Patterns](#range-patterns)
//!   - [Regex Patterns](#regex-patterns)
//!   - [Method Call Patterns](#method-call-patterns)
//! - [Data Types](#data-types)
//!   - [Collections (Vec/Slice)](#collections-vecslice)
//!   - [Tuples](#tuples)
//!   - [Enums (Option/Result/Custom)](#enums-optionresultcustom)
//!   - [Smart Pointers](#smart-pointers)
//! - [Error Messages](#error-messages)
//! - [Advanced Usage](#advanced-usage)
//!
//! # Quick Start
//!
//! Add to your `Cargo.toml`:
//!
//! ```toml
//! [dev-dependencies]
//! assert-struct = "0.1"
//! ```
//!
//! Basic example:
//!
//! ```rust
//! use assert_struct::assert_struct;
//!
//! #[derive(Debug)]
//! struct User {
//!     name: String,
//!     age: u32,
//!     email: String,
//! }
//!
//! let user = User {
//!     name: "Alice".to_string(),
//!     age: 30,
//!     email: "alice@example.com".to_string(),
//! };
//!
//! // Only check the fields you care about
//! assert_struct!(user, User {
//!     name: "Alice",
//!     age: 30,
//!     ..  // Ignore email
//! });
//! ```
//!
//! # Core Concepts
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
//! String fields work naturally with string literals:
//!
//! ```rust
//! # use assert_struct::assert_struct;
//! # #[derive(Debug)]
//! # struct Message { text: String, urgent: bool }
//! let msg = Message {
//!     text: "Hello world".to_string(),
//!     urgent: false,
//! };
//!
//! assert_struct!(msg, Message {
//!     text: "Hello world",  // No .to_string() needed!
//!     urgent: false,
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
//! # struct Order { customer: Customer, total: f64 }
//! # #[derive(Debug)]
//! # struct Customer { name: String, address: Address }
//! # #[derive(Debug)]
//! # struct Address { city: String, country: String }
//! # let order = Order {
//! #     customer: Customer {
//! #         name: "Bob".to_string(),
//! #         address: Address { city: "Paris".to_string(), country: "France".to_string() }
//! #     },
//! #     total: 99.99
//! # };
//! assert_struct!(order, Order {
//!     customer: Customer {
//!         name: "Bob",
//!         address: Address {
//!             city: "Paris",
//!             country: "France",
//!         },
//!     },
//!     total: 99.99,
//! });
//!
//! // Or with partial matching
//! assert_struct!(order, Order {
//!     customer: Customer {
//!         name: "Bob",
//!         address: Address { city: "Paris", .. },
//!         ..
//!     },
//!     ..
//! });
//! ```
//!
//! # Pattern Types
//!
//! ## Comparison Operators
//!
//! Use comparison operators for numeric assertions:
//!
//! ```rust
//! # use assert_struct::assert_struct;
//! # #[derive(Debug)]
//! # struct Metrics { cpu: f64, memory: u64, requests: u32 }
//! # let metrics = Metrics { cpu: 75.5, memory: 1024, requests: 150 };
//! assert_struct!(metrics, Metrics {
//!     cpu: < 80.0,          // Less than 80%
//!     memory: <= 2048,      // At most 2GB
//!     requests: > 100,      // More than 100
//! });
//! ```
//!
//! All comparison operators work: `<`, `<=`, `>`, `>=`
//!
//! ## Equality Operators
//!
//! Use explicit equality for clarity:
//!
//! ```rust
//! # use assert_struct::assert_struct;
//! # #[derive(Debug)]
//! # struct Status { code: i32, active: bool }
//! # let status = Status { code: 200, active: true };
//! assert_struct!(status, Status {
//!     code: == 200,         // Explicit equality
//!     active: != false,     // Not equal to false
//! });
//! ```
//!
//! ## Range Patterns
//!
//! Use ranges for boundary checks:
//!
//! ```rust
//! # use assert_struct::assert_struct;
//! # #[derive(Debug)]
//! # struct Person { age: u32, score: f64 }
//! # let person = Person { age: 25, score: 87.5 };
//! assert_struct!(person, Person {
//!     age: 18..=65,         // Working age range
//!     score: 0.0..100.0,    // Valid score range
//! });
//! ```
//!
//! ## Regex Patterns
//!
//! Match string patterns with regular expressions (requires `regex` feature, enabled by default):
//!
//! ```rust
//! # #[cfg(feature = "regex")]
//! # {
//! # use assert_struct::assert_struct;
//! # #[derive(Debug)]
//! # struct Account { username: String, email: String }
//! # let account = Account {
//! #     username: "alice_doe".to_string(),
//! #     email: "alice@company.com".to_string(),
//! # };
//! assert_struct!(account, Account {
//!     username: =~ r"^[a-z_]+$",        // Lowercase and underscores
//!     email: =~ r"@company\.com$",      // Company email domain
//! });
//! # }
//! ```
//!
//! ## Method Call Patterns
//!
//! Call methods on fields and assert on their results:
//!
//! ```rust
//! # use assert_struct::assert_struct;
//! # use std::collections::HashMap;
//! # #[derive(Debug)]
//! # struct Data {
//! #     content: String,
//! #     items: Vec<i32>,
//! #     metadata: Option<String>,
//! #     cache: HashMap<String, i32>,
//! # }
//! # let mut map = HashMap::new();
//! # map.insert("key1".to_string(), 42);
//! # let data = Data {
//! #     content: "hello world".to_string(),
//! #     items: vec![1, 2, 3, 4, 5],
//! #     metadata: Some("cached".to_string()),
//! #     cache: map,
//! # };
//! assert_struct!(data, Data {
//!     content.len(): 11,                    // String length
//!     items.len(): >= 5,                    // Vector size check
//!     metadata.is_some(): true,             // Option state
//!     cache.contains_key("key1"): true,     // HashMap lookup
//!     ..
//! });
//! ```
//!
//! Method calls work with arguments too:
//!
//! ```rust
//! # use assert_struct::assert_struct;
//! # #[derive(Debug)]
//! # struct Text { content: String, other: String }
//! # let text = Text { content: "hello world".to_string(), other: "test".to_string() };
//! assert_struct!(text, Text {
//!     content.starts_with("hello"): true,
//!     ..
//! });
//!
//! assert_struct!(text, Text {
//!     content.contains("world"): true,
//!     ..
//! });
//! ```
//!
//! # Data Types
//!
//! ## Collections (Vec/Slice)
//!
//! Element-wise pattern matching for vectors:
//!
//! ```rust
//! # use assert_struct::assert_struct;
//! # #[derive(Debug)]
//! # struct Data { values: Vec<i32>, names: Vec<String> }
//! # let data = Data {
//! #     values: vec![5, 15, 25],
//! #     names: vec!["alice".to_string(), "bob".to_string()],
//! # };
//! // Exact matching
//! assert_struct!(data, Data {
//!     values: [5, 15, 25],
//!     names: ["alice", "bob"],  // String literals work in slices too!
//! });
//!
//! // Pattern matching for each element
//! assert_struct!(data, Data {
//!     values: [> 0, < 20, >= 25],    // Different pattern per element
//!     names: ["alice", "bob"],
//! });
//! ```
//!
//! Partial slice matching:
//!
//! ```rust
//! # use assert_struct::assert_struct;
//! # #[derive(Debug)]
//! # struct Data { items: Vec<i32> }
//! # let data = Data { items: vec![1, 2, 3, 4, 5] };
//! assert_struct!(data, Data {
//!     items: [1, 2, ..],      // First two elements, ignore rest
//! });
//!
//! assert_struct!(data, Data {
//!     items: [.., 4, 5],      // Last two elements
//! });
//!
//! assert_struct!(data, Data {
//!     items: [1, .., 5],      // First and last elements
//! });
//! ```
//!
//! ## Tuples
//!
//! Full support for multi-field tuples:
//!
//! ```rust
//! # use assert_struct::assert_struct;
//! # #[derive(Debug)]
//! # struct Data { point: (i32, i32), metadata: (String, u32, bool) }
//! # let data = Data {
//! #     point: (15, 25),
//! #     metadata: ("info".to_string(), 100, true),
//! # };
//! // Basic tuple matching
//! assert_struct!(data, Data {
//!     point: (15, 25),
//!     metadata: ("info", 100, true),  // String literals work in tuples!
//! });
//!
//! // Advanced patterns
//! assert_struct!(data, Data {
//!     point: (> 10, < 30),           // Comparison operators
//!     metadata: ("info", >= 50, true),
//! });
//! ```
//!
//! Tuple method calls:
//!
//! ```rust
//! # use assert_struct::assert_struct;
//! # #[derive(Debug)]
//! # struct Data { coords: (String, Vec<i32>) }
//! # let data = Data {
//! #     coords: ("location".to_string(), vec![1, 2, 3]),
//! # };
//! assert_struct!(data, Data {
//!     coords: (0.len(): 8, 1.len(): 3),  // Method calls on tuple elements
//! });
//! ```
//!
//! ## Enums (Option/Result/Custom)
//!
//! ### Option Types
//!
//! ```rust
//! # use assert_struct::assert_struct;
//! # #[derive(Debug)]
//! # struct User { name: Option<String>, age: Option<u32> }
//! # let user = User { name: Some("Alice".to_string()), age: Some(30) };
//! assert_struct!(user, User {
//!     name: Some("Alice"),
//!     age: Some(30),
//! });
//!
//! // Advanced patterns inside Option
//! assert_struct!(user, User {
//!     name: Some("Alice"),
//!     age: Some(>= 18),      // Adult check inside Some
//! });
//! ```
//!
//! ### Result Types
//!
//! ```rust
//! # use assert_struct::assert_struct;
//! # #[derive(Debug)]
//! # struct Response { result: Result<String, String> }
//! # let response = Response { result: Ok("success".to_string()) };
//! assert_struct!(response, Response {
//!     result: Ok("success"),
//! });
//!
//! // Pattern matching inside Result
//! # let response = Response { result: Ok("user123".to_string()) };
//! assert_struct!(response, Response {
//!     result: Ok(=~ r"^user\d+$"),  // Regex inside Ok
//! });
//! ```
//!
//! ### Custom Enums
//!
//! ```rust
//! # use assert_struct::assert_struct;
//! # #[derive(Debug, PartialEq)]
//! # enum Status { Active, Pending { since: String } }
//! # #[derive(Debug)]
//! # struct Account { status: Status }
//! # let account = Account { status: Status::Pending { since: "2024-01-01".to_string() } };
//! // Unit variants
//! let active_account = Account { status: Status::Active };
//! assert_struct!(active_account, Account {
//!     status: Status::Active,
//! });
//!
//! // Struct variants with partial matching
//! assert_struct!(account, Account {
//!     status: Status::Pending { since: "2024-01-01" },
//! });
//! ```
//!
//! ## Smart Pointers
//!
//! Dereference smart pointers directly in patterns:
//!
//! ```rust
//! # use assert_struct::assert_struct;
//! # use std::rc::Rc;
//! # use std::sync::Arc;
//! # #[derive(Debug)]
//! # struct Cache {
//! #     data: Arc<String>,
//! #     count: Box<i32>,
//! #     shared: Rc<bool>,
//! # }
//! # let cache = Cache {
//! #     data: Arc::new("cached".to_string()),
//! #     count: Box::new(42),
//! #     shared: Rc::new(true),
//! # };
//! assert_struct!(cache, Cache {
//!     *data: "cached",       // Dereference Arc<String>
//!     *count: > 40,          // Dereference Box<i32> with comparison
//!     *shared: true,         // Dereference Rc<bool>
//! });
//! ```
//!
//! Multiple dereferencing for nested pointers:
//!
//! ```rust
//! # use assert_struct::assert_struct;
//! # #[derive(Debug)]
//! # struct Nested { value: Box<Box<i32>> }
//! # let nested = Nested { value: Box::new(Box::new(42)) };
//! assert_struct!(nested, Nested {
//!     **value: 42,           // Double dereference
//! });
//! ```
//!
//! ## Wildcard Patterns
//!
//! Use wildcard patterns (`_`) to avoid importing types while still asserting on their structure:
//!
//! ```rust
//! # use assert_struct::assert_struct;
//! # mod api {
//! #     #[derive(Debug)]
//! #     pub struct Response {
//! #         pub user: User,
//! #         pub metadata: Metadata,
//! #     }
//! #     #[derive(Debug)]
//! #     pub struct User {
//! #         pub id: u32,
//! #         pub name: String,
//! #     }
//! #     #[derive(Debug)]
//! #     pub struct Metadata {
//! #         pub timestamp: u64,
//! #         pub version: String,
//! #     }
//! # }
//! # let response = api::Response {
//! #     user: api::User { id: 123, name: "Alice".to_string() },
//! #     metadata: api::Metadata { timestamp: 1234567890, version: "1.0".to_string() }
//! # };
//! // No need to import User or Metadata types!
//! assert_struct!(response, _ {
//!     user: _ {
//!         id: 123,
//!         name: "Alice",
//!         ..
//!     },
//!     metadata: _ {
//!         version: "1.0",
//!         ..  // Ignore other metadata fields
//!     },
//!     ..
//! });
//! ```
//!
//! This is particularly useful when testing API responses where you don't want to import all the nested types:
//!
//! ```rust
//! # use assert_struct::assert_struct;
//! # #[derive(Debug)]
//! # struct JsonResponse { data: Data }
//! # #[derive(Debug)]
//! # struct Data { items: Vec<Item>, total: u32 }
//! # #[derive(Debug)]
//! # struct Item { id: u32, value: String }
//! # let json_response = JsonResponse {
//! #     data: Data {
//! #         items: vec![Item { id: 1, value: "test".to_string() }],
//! #         total: 1
//! #     }
//! # };
//! // Test deeply nested structures without imports
//! assert_struct!(json_response, _ {
//!     data: _ {
//!         items: [_ { id: 1, value: "test", .. }],
//!         total: 1,
//!         ..
//!     },
//!     ..
//! });
//! ```
//!
//! # Error Messages
//!
//! When assertions fail, `assert-struct` provides detailed, actionable error messages:
//!
//! ## Basic Mismatch
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
//! //   --> `user.name` (src/lib.rs:456)
//! //   actual: "Alice"
//! //   expected: "Bob"
//! ```
//!
//! ## Comparison Failure
//!
//! ```rust,should_panic
//! # use assert_struct::assert_struct;
//! # #[derive(Debug)]
//! # struct Stats { score: u32 }
//! # let stats = Stats { score: 50 };
//! assert_struct!(stats, Stats {
//!     score: > 100,  // This will fail
//! });
//! // Error output:
//! // assert_struct! failed:
//! //
//! // comparison mismatch:
//! //   --> `stats.score` (src/lib.rs:469)
//! //   actual: 50
//! //   expected: > 100
//! ```
//!
//! ## Nested Field Errors
//!
//! Error messages show the exact path to the failing field, even in deeply nested structures.
//! Method calls are also shown in the field path for clear debugging.
//!
//! # Advanced Usage
//!
//! ## Pattern Composition
//!
//! Combine multiple patterns for comprehensive assertions:
//!
//! ```rust
//! # use assert_struct::assert_struct;
//! # #[derive(Debug)]
//! # struct Complex {
//! #     data: Option<Vec<i32>>,
//! #     metadata: (String, u32),
//! # }
//! # let complex = Complex {
//! #     data: Some(vec![1, 2, 3]),
//! #     metadata: ("info".to_string(), 42),
//! # };
//! assert_struct!(complex, Complex {
//!     data: Some([> 0, > 1, > 2]),              // Option + Vec + comparisons
//!     metadata: ("info", > 40),                 // Tuple + string + comparison
//!     ..
//! });
//!
//! // Verify data length separately
//! assert_eq!(complex.data.as_ref().unwrap().len(), 3);
//! ```
//!
//! ## Real-World Testing Patterns
//!
//! See the [examples directory](../../examples/) for comprehensive real-world examples including:
//! - API response validation
//! - Database record testing
//! - Configuration validation
//! - Event system testing
//!
//! For complete specification details, see the [`assert_struct!`] macro documentation.

// Re-export the procedural macro
pub use assert_struct_macros::assert_struct;

// Error handling module
#[doc(hidden)]
pub mod error;

// Hidden module for macro support functions
#[doc(hidden)]
pub mod __macro_support {
    pub use crate::error::{ErrorContext, ErrorType, PatternNode, format_errors_with_root};

    /// Helper function to enable type inference for closure parameters in assert_struct patterns
    #[inline]
    pub fn check_closure_condition<T, F>(value: T, predicate: F) -> bool
    where
        F: FnOnce(T) -> bool,
    {
        predicate(value)
    }
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
