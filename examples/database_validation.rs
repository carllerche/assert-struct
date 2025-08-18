//! Database Record Validation
//!
//! This example demonstrates how to use `assert-struct` to validate database records
//! and query results in real-world applications. This covers common scenarios when
//! testing database operations, ORM queries, and data consistency.
//!
//! The example covers:
//! - Database entity validation after queries
//! - Relationship testing (foreign keys, joins)
//! - Aggregate query results
//! - Data transformation validation
//! - Audit trail and timestamp validation
//! - Business rule enforcement in data

use assert_struct::assert_struct;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// User entity from database
#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub status: UserStatus,
    pub email_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub login_count: i32,
    pub profile: Option<UserProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
enum UserStatus {
    Active,
    Inactive,
    Suspended,
    Deleted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserProfile {
    pub user_id: i64,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub website: Option<String>,
    pub location: Option<String>,
    pub birth_date: Option<DateTime<Utc>>,
    pub preferences: UserPreferences,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserPreferences {
    pub theme: String,
    pub language: String,
    pub timezone: String,
    pub email_notifications: bool,
    pub privacy_level: i32, // 1=public, 2=friends, 3=private
}

/// Order entity with relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Order {
    pub id: i64,
    pub user_id: i64,
    pub order_number: String,
    pub status: OrderStatus,
    pub currency: String,
    pub subtotal: rust_decimal::Decimal,
    pub tax_amount: rust_decimal::Decimal,
    pub shipping_amount: rust_decimal::Decimal,
    pub total_amount: rust_decimal::Decimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub shipped_at: Option<DateTime<Utc>>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub items: Vec<OrderItem>,
    pub shipping_address: Address,
    pub billing_address: Option<Address>,
    pub payment_method: PaymentMethod,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
enum OrderStatus {
    Pending,
    Confirmed,
    Processing,
    Shipped,
    Delivered,
    Cancelled,
    Refunded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OrderItem {
    pub id: i64,
    pub order_id: i64,
    pub product_id: i64,
    pub product_name: String,
    pub quantity: i32,
    pub unit_price: rust_decimal::Decimal,
    pub total_price: rust_decimal::Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Address {
    pub street: String,
    pub city: String,
    pub state: Option<String>,
    pub postal_code: String,
    pub country: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PaymentMethod {
    pub method_type: String, // "card", "paypal", etc.
    pub last_four: Option<String>, // For cards
    pub provider: String,
}

/// Query result aggregations
#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserAnalytics {
    pub user_id: i64,
    pub total_orders: i64,
    pub total_spent: rust_decimal::Decimal,
    pub average_order_value: rust_decimal::Decimal,
    pub last_order_date: Option<DateTime<Utc>>,
    pub favorite_categories: Vec<String>,
    pub order_frequency_days: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SalesReport {
    pub period: String,
    pub total_revenue: rust_decimal::Decimal,
    pub total_orders: i64,
    pub unique_customers: i64,
    pub average_order_value: rust_decimal::Decimal,
    pub top_products: Vec<ProductSales>,
    pub revenue_by_day: HashMap<String, rust_decimal::Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProductSales {
    pub product_id: i64,
    pub product_name: String,
    pub units_sold: i64,
    pub revenue: rust_decimal::Decimal,
}

// Mock database functions for testing
fn mock_create_user() -> User {
    User {
        id: 1,
        username: "john_doe".to_string(),
        email: "john@example.com".to_string(),
        password_hash: "$2b$12$abc123...".to_string(),
        first_name: Some("John".to_string()),
        last_name: Some("Doe".to_string()),
        status: UserStatus::Active,
        email_verified: false, // New user
        created_at: Utc::now(),
        updated_at: Utc::now(),
        last_login: None,
        login_count: 0,
        profile: None, // Created separately
    }
}

fn mock_user_with_profile() -> User {
    let mut user = mock_create_user();
    user.id = 2;
    user.username = "alice_dev".to_string();
    user.email = "alice@example.com".to_string();
    user.email_verified = true;
    user.login_count = 15;
    user.last_login = Some(Utc::now() - chrono::Duration::hours(2));
    user.profile = Some(UserProfile {
        user_id: 2,
        bio: Some("Software developer passionate about Rust".to_string()),
        avatar_url: Some("https://example.com/avatars/alice.jpg".to_string()),
        website: Some("https://alice.dev".to_string()),
        location: Some("San Francisco, CA".to_string()),
        birth_date: Some(DateTime::parse_from_rfc3339("1990-05-15T00:00:00Z").unwrap().with_timezone(&Utc)),
        preferences: UserPreferences {
            theme: "dark".to_string(),
            language: "en".to_string(),
            timezone: "America/Los_Angeles".to_string(),
            email_notifications: true,
            privacy_level: 2,
        },
    });
    user
}

fn mock_completed_order() -> Order {
    use rust_decimal_macros::dec;
    
    Order {
        id: 1001,
        user_id: 2,
        order_number: "ORD-2024-001001".to_string(),
        status: OrderStatus::Delivered,
        currency: "USD".to_string(),
        subtotal: dec!(89.97),
        tax_amount: dec!(7.20),
        shipping_amount: dec!(12.99),
        total_amount: dec!(110.16),
        created_at: Utc::now() - chrono::Duration::days(7),
        updated_at: Utc::now() - chrono::Duration::days(1),
        shipped_at: Some(Utc::now() - chrono::Duration::days(3)),
        delivered_at: Some(Utc::now() - chrono::Duration::days(1)),
        items: vec![
            OrderItem {
                id: 1,
                order_id: 1001,
                product_id: 501,
                product_name: "Premium Headphones".to_string(),
                quantity: 1,
                unit_price: dec!(79.99),
                total_price: dec!(79.99),
            },
            OrderItem {
                id: 2,
                order_id: 1001,
                product_id: 502,
                product_name: "USB Cable".to_string(),
                quantity: 1,
                unit_price: dec!(9.98),
                total_price: dec!(9.98),
            },
        ],
        shipping_address: Address {
            street: "123 Main St".to_string(),
            city: "San Francisco".to_string(),
            state: Some("CA".to_string()),
            postal_code: "94105".to_string(),
            country: "US".to_string(),
        },
        billing_address: None, // Same as shipping
        payment_method: PaymentMethod {
            method_type: "card".to_string(),
            last_four: Some("4242".to_string()),
            provider: "stripe".to_string(),
        },
    }
}

fn mock_user_analytics() -> UserAnalytics {
    use rust_decimal_macros::dec;
    
    UserAnalytics {
        user_id: 2,
        total_orders: 5,
        total_spent: dec!(445.78),
        average_order_value: dec!(89.16),
        last_order_date: Some(Utc::now() - chrono::Duration::days(7)),
        favorite_categories: vec!["Electronics".to_string(), "Books".to_string()],
        order_frequency_days: 30.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    /// Test new user creation with validation
    #[test]
    fn test_new_user_creation() {
        let user = mock_create_user();

        assert_struct!(user, User {
            // Database-generated fields
            id: > 0,                                          // Valid ID assigned
            
            // Required fields validation  
            username: =~ r"^[a-z][a-z0-9_]{2,20}$",          // Valid username format
            email: =~ r"^[^@]+@[^@]+\.[^@]+$",               // Basic email validation
            password_hash: =~ r"^\$2b\$\d{2}\$",             // bcrypt hash format
            
            // New user state validation
            status: UserStatus::Active,                        // Default status
            email_verified: false,                            // Needs verification
            login_count: 0,                                   // No logins yet
            last_login: None,                                 // Never logged in
            profile: None,                                    // No profile yet
            
            // Audit fields
            created_at: _, // Present but value doesn't matter for new users
            updated_at: _, // Present but value doesn't matter for new users
        });

        // Business rule validation
        assert!(user.created_at <= Utc::now(), "Created time should not be in future");
        assert_eq!(user.created_at, user.updated_at, "New user timestamps should match");
    }

    /// Test user profile creation and relationships
    #[test]
    fn test_user_with_profile() {
        let user = mock_user_with_profile();

        assert_struct!(user, User {
            id: > 0,
            username: "alice_dev",
            email_verified: true,                             // Verified user
            login_count: > 0,                                 // Has logged in
            last_login: Some(_),                              // Recent login
            
            // Profile relationship validation
            profile: Some(UserProfile {
                user_id: user.id,                             // Foreign key matches
                bio: Some(_),                                 // Has bio
                website: Some(=~ r"^https?://.*"),            // Valid URL format
                location: Some(_),                            // Has location
                
                preferences: UserPreferences {
                    theme: =~ r"^(light|dark|auto)$",         // Valid theme
                    language: =~ r"^[a-z]{2}$",               // ISO language code
                    timezone: =~ r"^[A-Za-z_]+/[A-Za-z_]+$",  // Valid timezone format
                    privacy_level: 1..=3,                     // Valid privacy range
                    ..
                },
                ..
            }),
            
            // Active user characteristics
            status: UserStatus::Active,
            ..
        });
    }

    /// Test order creation with financial calculations
    #[test]
    fn test_order_financial_validation() {
        let order = mock_completed_order();

        assert_struct!(order, Order {
            id: > 0,
            user_id: > 0,                                     // Valid user reference
            order_number: =~ r"^ORD-\d{4}-\d{6}$",           // Order number format
            
            // Financial validation
            currency: =~ r"^[A-Z]{3}$",                       // ISO currency code
            subtotal: > dec!(0),                              // Positive subtotal
            tax_amount: >= dec!(0),                           // Non-negative tax
            shipping_amount: >= dec!(0),                      // Non-negative shipping
            total_amount: > dec!(0),                          // Positive total
            
            // Order lifecycle
            status: OrderStatus::Delivered,                   // Completed order
            shipped_at: Some(_),                              // Has shipping date
            delivered_at: Some(_),                            // Has delivery date
            
            // Order items validation
            items.len(): > 0,                                 // Has items
            items: [
                OrderItem {
                    order_id: order.id,                       // Foreign key consistency
                    quantity: > 0,                            // Positive quantity
                    unit_price: > dec!(0),                    // Valid price
                    total_price: > dec!(0),                   // Valid total
                    ..
                },
                ..                                            // May have more items
            ],
            
            // Address validation
            shipping_address: Address {
                street.len(): > 0,                            // Non-empty address
                city.len(): > 0,                              // Non-empty city
                postal_code.len(): > 0,                       // Has postal code
                country: =~ r"^[A-Z]{2}$",                    // ISO country code
                ..
            },
            
            // Payment method validation
            payment_method: PaymentMethod {
                method_type: =~ r"^(card|paypal|bank)$",      // Valid payment type
                provider.len(): > 0,                          // Has provider
                ..
            },
            
            ..
        });

        // Business rule validation: total calculation
        let expected_total = order.subtotal + order.tax_amount + order.shipping_amount;
        assert_eq!(order.total_amount, expected_total, 
                  "Total amount should equal subtotal + tax + shipping");

        // Order timeline validation
        assert!(order.created_at <= order.updated_at, 
                "Updated time should be >= created time");
        
        if let (Some(shipped), Some(delivered)) = (order.shipped_at, order.delivered_at) {
            assert!(shipped <= delivered, "Shipping should occur before delivery");
            assert!(order.created_at <= shipped, "Order should be created before shipping");
        }

        // Item total validation
        let items_subtotal: rust_decimal::Decimal = order.items.iter()
            .map(|item| item.total_price)
            .sum();
        assert_eq!(order.subtotal, items_subtotal, 
                  "Order subtotal should equal sum of item totals");
    }

    /// Test user analytics aggregation
    #[test]
    fn test_user_analytics_aggregation() {
        let analytics = mock_user_analytics();

        assert_struct!(analytics, UserAnalytics {
            user_id: > 0,                                     // Valid user ID
            
            // Aggregate metrics validation
            total_orders: > 0,                                // Has orders
            total_spent: > dec!(0),                           // Has spent money
            average_order_value: > dec!(0),                   // Positive AOV
            last_order_date: Some(_),                         // Recent activity
            
            // Behavioral data
            favorite_categories.len(): > 0,                   // Has preferences
            order_frequency_days: > 0.0,                      // Positive frequency
        });

        // Business metrics validation
        let expected_aov = analytics.total_spent / rust_decimal::Decimal::from(analytics.total_orders);
        assert_eq!(analytics.average_order_value, expected_aov, 
                  "Average order value calculation should be correct");

        // Category analysis
        assert!(analytics.favorite_categories.len() <= 5, 
                "Should limit to top categories");
        assert!(analytics.favorite_categories.contains(&"Electronics".to_string()), 
                "Expected favorite category missing");
    }

    /// Test data consistency across relationships
    #[test]
    fn test_user_order_relationship_consistency() {
        let user = mock_user_with_profile();
        let order = mock_completed_order();
        let analytics = mock_user_analytics();

        // Ensure all records refer to the same user
        assert_eq!(user.id, order.user_id, "Order should belong to user");
        assert_eq!(user.id, analytics.user_id, "Analytics should be for same user");
        
        if let Some(ref profile) = user.profile {
            assert_eq!(profile.user_id, user.id, "Profile should belong to user");
        }

        // Cross-validation
        assert_struct!(user, User {
            id: order.user_id,                               // Consistent user ID
            status: UserStatus::Active,                      // Active users can place orders
            email_verified: true,                            // Verified users preferred
            ..
        });

        assert_struct!(analytics, UserAnalytics {
            user_id: user.id,                               // Same user
            total_orders: > 0,                              // User has order history
            ..
        });
    }

    /// Test audit trail validation
    #[test]
    fn test_audit_trail_validation() {
        let user = mock_user_with_profile();
        let order = mock_completed_order();

        // User audit validation
        assert_struct!(user, User {
            created_at: _, 
            updated_at: _,
            last_login: Some(_),                            // Active user
            login_count: > 0,                               // Has login history
            ..
        });

        // Order audit validation  
        assert_struct!(order, Order {
            created_at: _,
            updated_at: _,
            status: OrderStatus::Delivered,                 // Final status
            shipped_at: Some(_),                            // Tracking info
            delivered_at: Some(_),                          // Completion info
            ..
        });

        // Timeline consistency
        assert!(user.created_at < order.created_at, 
                "User should be created before placing orders");
        
        if let Some(last_login) = user.last_login {
            // User should have logged in around order time (within 30 days)
            let order_window = chrono::Duration::days(30);
            let login_window = (order.created_at - order_window, order.created_at + order_window);
            assert!(last_login >= login_window.0 && last_login <= login_window.1,
                    "User login should be near order time");
        }
    }

    /// Test data privacy and security validation
    #[test]
    fn test_data_privacy_validation() {
        let user = mock_user_with_profile();

        assert_struct!(user, User {
            // Sensitive fields should not be exposed
            password_hash: =~ r"^\$2b\$",                    // Hash format, not plaintext
            email: =~ r"^[^@]+@[^@]+\.[^@]+$",              // Email format validation
            
            profile: Some(UserProfile {
                preferences: UserPreferences {
                    privacy_level: 1..=3,                   // Valid privacy setting
                    ..
                },
                ..
            }),
            ..
        });

        // Ensure password is properly hashed
        assert!(user.password_hash.len() > 50, "Password hash should be substantial");
        assert!(!user.password_hash.contains("password"), "Should not contain plaintext");

        // Privacy level enforcement
        if let Some(ref profile) = user.profile {
            let privacy = profile.preferences.privacy_level;
            match privacy {
                1 => { /* Public profile - all fields allowed */ },
                2 => { /* Friends only - location ok but not birth_date in public APIs */ },
                3 => { 
                    // Private profile - minimal info
                    assert!(profile.bio.is_none() || 
                           profile.bio.as_ref().unwrap().len() < 100, 
                           "Private profiles should have limited bio");
                },
                _ => panic!("Invalid privacy level"),
            }
        }
    }
}

/// Integration tests for database operations
#[cfg(test)]
mod integration_tests {
    use super::*;

    // Mock database operations for integration testing
    struct UserRepository;

    impl UserRepository {
        async fn create_user(&self, username: &str, email: &str, password: &str) -> Result<User, &'static str> {
            if username.len() < 3 { return Err("Username too short"); }
            if !email.contains('@') { return Err("Invalid email"); }
            if password.len() < 8 { return Err("Password too short"); }
            
            Ok(mock_create_user())
        }

        async fn find_by_id(&self, id: i64) -> Result<Option<User>, &'static str> {
            match id {
                1 => Ok(Some(mock_create_user())),
                2 => Ok(Some(mock_user_with_profile())),
                _ => Ok(None),
            }
        }

        async fn update_login_info(&self, user_id: i64) -> Result<User, &'static str> {
            let mut user = mock_user_with_profile();
            user.id = user_id;
            user.last_login = Some(Utc::now());
            user.login_count += 1;
            user.updated_at = Utc::now();
            Ok(user)
        }
    }

    #[tokio::test]
    async fn test_user_creation_flow() {
        let repo = UserRepository;
        
        let user = repo.create_user("testuser", "test@example.com", "password123").await.unwrap();

        // Validate created user matches expected structure
        assert_struct!(user, User {
            id: > 0,                                          // Database assigned ID
            username: "john_doe",                             // From mock (would be "testuser" in real impl)
            email: =~ r"^[^@]+@[^@]+\.[^@]+$",               // Valid email
            status: UserStatus::Active,                       // Default for new users
            email_verified: false,                            // New users unverified
            login_count: 0,                                   // No logins yet
            last_login: None,                                 // Never logged in
            created_at: _,                                    // Has creation timestamp
            updated_at: _,                                    // Has update timestamp
            profile: None,                                    // No profile initially
            ..
        });
    }

    #[tokio::test]
    async fn test_user_login_update() {
        let repo = UserRepository;
        
        let user = repo.update_login_info(2).await.unwrap();

        // Validate login info was updated correctly
        assert_struct!(user, User {
            id: 2,
            last_login: Some(_),                              // Login time recorded
            login_count: > 0,                                 // Count incremented
            updated_at: _,                                    // Update timestamp refreshed
            ..
        });

        // Validate timestamp recency (within last minute)
        if let Some(last_login) = user.last_login {
            let time_diff = Utc::now().signed_duration_since(last_login);
            assert!(time_diff.num_seconds() < 60, "Last login should be very recent");
        }

        assert!(user.updated_at > user.created_at, "Updated time should be after created time");
    }

    #[tokio::test]
    async fn test_user_not_found() {
        let repo = UserRepository;
        
        let result = repo.find_by_id(99999).await.unwrap();

        // Should return None for non-existent user
        assert_eq!(result, None);
    }
}

fn main() {
    println!("Database Validation Example");
    println!("Run with: cargo test --example database_validation");
    
    // Example of validating a query result in application code
    let user = mock_user_with_profile();
    
    // Quick validation that user has required fields for business logic
    assert_struct!(user, User {
        id: > 0,
        status: UserStatus::Active,
        email_verified: true,                               // Required for certain operations
        profile: Some(UserProfile {
            preferences: UserPreferences {
                privacy_level: 1..=3,                       // Valid privacy setting
                ..
            },
            ..
        }),
        ..
    });
    
    println!("User validation passed: ID {}, Username: {}", user.id, user.username);
}