//! Ergonomic structural assertions for Rust tests.
//!
//! `assert-struct` is a procedural macro that enables clean, readable assertions for complex
//! data structures without verbose field-by-field comparisons. It's the testing tool you need
//! when `assert_eq!` isn't enough and manually comparing fields is too cumbersome.
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

// Future: Like trait will be defined here
// pub trait Like<Rhs = Self> {
//     fn like(&self, other: &Rhs) -> bool;
// }