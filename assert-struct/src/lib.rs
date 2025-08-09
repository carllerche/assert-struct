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

// Re-export the procedural macro
pub use assert_struct_macros::assert_struct;

// Future: Like trait will be defined here
// pub trait Like<Rhs = Self> {
//     fn like(&self, other: &Rhs) -> bool;
// }