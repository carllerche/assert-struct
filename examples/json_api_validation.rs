//! JSON API Response Validation
//!
//! This example demonstrates how to use `assert-struct` to validate complex JSON API responses
//! after deserialization. This is one of the most common use cases for structural assertions
//! in real-world testing scenarios.
//!
//! The example covers:
//! - Nested API response structures
//! - Partial matching for ignoring volatile fields
//! - Method calls for derived properties
//! - Collection validation with patterns
//! - Error handling structures
//! - Mixed data types and validation rules

use assert_struct::assert_struct;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// User profile data from API
#[derive(Debug, Serialize, Deserialize)]
struct UserProfile {
    pub id: u64,
    pub username: String,
    pub email: String,
    pub full_name: Option<String>,
    pub avatar_url: Option<String>,
    pub created_at: String, // ISO 8601 timestamp
    pub updated_at: String,
    pub is_verified: bool,
    pub subscription: SubscriptionInfo,
    pub preferences: UserPreferences,
    pub stats: UserStats,
}

#[derive(Debug, Serialize, Deserialize)]
struct SubscriptionInfo {
    pub tier: String,
    pub expires_at: Option<String>,
    pub features: Vec<String>,
    pub billing: BillingInfo,
}

#[derive(Debug, Serialize, Deserialize)]
struct BillingInfo {
    pub currency: String,
    pub amount: f64,
    pub interval: String, // "monthly", "yearly", etc.
    pub next_billing_date: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct UserPreferences {
    pub theme: String,
    pub notifications: NotificationSettings,
    pub privacy: PrivacySettings,
}

#[derive(Debug, Serialize, Deserialize)]
struct NotificationSettings {
    pub email_notifications: bool,
    pub push_notifications: bool,
    pub marketing_emails: bool,
    pub frequency: String, // "immediate", "daily", "weekly"
}

#[derive(Debug, Serialize, Deserialize)]
struct PrivacySettings {
    pub profile_visibility: String, // "public", "private", "friends"
    pub search_indexing: bool,
    pub data_sharing: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct UserStats {
    pub posts_count: u32,
    pub followers_count: u32,
    pub following_count: u32,
    pub likes_received: u32,
    pub join_date_days_ago: u32,
}

/// API response wrapper with metadata
#[derive(Debug, Serialize, Deserialize)]
struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ApiError>,
    pub meta: ResponseMeta,
}

#[derive(Debug, Serialize, Deserialize)]
struct ApiError {
    pub code: String,
    pub message: String,
    pub details: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ResponseMeta {
    pub timestamp: String,
    pub request_id: String,
    pub version: String,
    pub rate_limit: RateLimit,
}

#[derive(Debug, Serialize, Deserialize)]
struct RateLimit {
    pub remaining: u32,
    pub reset_at: String,
    pub total: u32,
}

/// Example API responses for testing
fn mock_successful_user_response() -> ApiResponse<UserProfile> {
    serde_json::from_str(r#"
    {
        "success": true,
        "data": {
            "id": 12345,
            "username": "alice_dev",
            "email": "alice@example.com",
            "full_name": "Alice Developer",
            "avatar_url": "https://api.example.com/avatars/12345.jpg",
            "created_at": "2023-01-15T10:30:00Z",
            "updated_at": "2024-01-10T14:22:33Z",
            "is_verified": true,
            "subscription": {
                "tier": "premium",
                "expires_at": "2024-12-31T23:59:59Z",
                "features": ["advanced_analytics", "priority_support", "custom_themes"],
                "billing": {
                    "currency": "USD",
                    "amount": 29.99,
                    "interval": "monthly",
                    "next_billing_date": "2024-02-15T10:30:00Z"
                }
            },
            "preferences": {
                "theme": "dark",
                "notifications": {
                    "email_notifications": true,
                    "push_notifications": false,
                    "marketing_emails": true,
                    "frequency": "daily"
                },
                "privacy": {
                    "profile_visibility": "public",
                    "search_indexing": true,
                    "data_sharing": false
                }
            },
            "stats": {
                "posts_count": 156,
                "followers_count": 2847,
                "following_count": 423,
                "likes_received": 12950,
                "join_date_days_ago": 374
            }
        },
        "error": null,
        "meta": {
            "timestamp": "2024-01-10T15:30:45Z",
            "request_id": "req_abc123def456",
            "version": "v1.2.3",
            "rate_limit": {
                "remaining": 98,
                "reset_at": "2024-01-10T16:00:00Z",
                "total": 100
            }
        }
    }
    "#).unwrap()
}

fn mock_error_response() -> ApiResponse<UserProfile> {
    serde_json::from_str(r#"
    {
        "success": false,
        "data": null,
        "error": {
            "code": "USER_NOT_FOUND",
            "message": "The requested user could not be found",
            "details": {
                "user_id": "99999",
                "suggestion": "Check if the user ID is correct"
            }
        },
        "meta": {
            "timestamp": "2024-01-10T15:30:45Z",
            "request_id": "req_error123",
            "version": "v1.2.3",
            "rate_limit": {
                "remaining": 97,
                "reset_at": "2024-01-10T16:00:00Z",
                "total": 100
            }
        }
    }
    "#).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test successful API response with comprehensive validation
    #[test]
    fn test_successful_user_profile_response() {
        let response = mock_successful_user_response();

        assert_struct!(response, ApiResponse {
            success: true,
            data: Some(UserProfile {
                // Essential user data validation
                id: > 0,                                    // Valid user ID
                username: =~ r"^[a-z][a-z0-9_]*$",         // Valid username format
                email: =~ r"^[^@]+@[^@]+\.[^@]+$",         // Basic email validation
                full_name: Some("Alice Developer"),         // Full name present
                is_verified: true,                          // User is verified
                
                // Subscription validation
                subscription: SubscriptionInfo {
                    tier: =~ r"^(free|premium|enterprise)$",   // Valid tier
                    features.len(): > 0,                       // Has features
                    features: [
                        =~ r"advanced_analytics",              // Has analytics
                        =~ r".*support.*",                     // Has some support
                        ..                                     // May have other features
                    ],
                    billing: BillingInfo {
                        currency: =~ r"^[A-Z]{3}$",           // ISO currency code
                        amount: > 0.0,                         // Positive amount
                        interval: =~ r"^(monthly|yearly)$",    // Valid interval
                        next_billing_date: Some(=~ r"^\d{4}-\d{2}-\d{2}T.*Z$"), // ISO date
                    },
                    ..
                },
                
                // User preferences validation
                preferences: UserPreferences {
                    theme: =~ r"^(light|dark|auto)$",          // Valid theme
                    notifications: NotificationSettings {
                        frequency: =~ r"^(immediate|daily|weekly)$", // Valid frequency
                        ..                                     // Other settings flexible
                    },
                    privacy: PrivacySettings {
                        profile_visibility: =~ r"^(public|private|friends)$", // Valid visibility
                        ..
                    },
                },
                
                // Statistics validation
                stats: UserStats {
                    posts_count: >= 0,                         // Non-negative stats
                    followers_count: >= 0,
                    following_count: >= 0,
                    likes_received: >= 0,
                    join_date_days_ago: > 0,                   // Account has some age
                },
                
                // Ignore volatile timestamp fields
                ..
            }),
            
            error: None,                                       // No error in success case
            
            // Meta validation with method calls
            meta: ResponseMeta {
                request_id: =~ r"^req_[a-f0-9]+$",            // Valid request ID format
                version: =~ r"^v\d+\.\d+\.\d+$",              // Semantic version
                request_id.len(): > 10,                        // Reasonable ID length
                rate_limit: RateLimit {
                    remaining: <= 100,                         // Within rate limit
                    total: 100,                                // Expected total
                    ..
                },
                ..                                             // Ignore timestamp
            },
        });
    }

    /// Test error response structure
    #[test]
    fn test_error_response_structure() {
        let response = mock_error_response();

        assert_struct!(response, ApiResponse::<UserProfile> {
            success: false,                                    // Error case
            data: None,                                        // No data on error
            
            error: Some(ApiError {
                code: =~ r"^[A-Z_]+$",                        // Error code format
                message.len(): > 10,                          // Meaningful message
                message.contains("user"): true,               // Error relates to user
                details: Some(_),                             // Has error details
            }),
            
            meta: ResponseMeta {
                request_id: =~ r"^req_.*$",                   // Has request ID
                rate_limit: RateLimit {
                    remaining: < 100,                          // Rate limit decremented
                    ..
                },
                ..
            },
        });
    }

    /// Test rate limiting behavior
    #[test]
    fn test_rate_limit_validation() {
        let response = mock_successful_user_response();

        // Focus specifically on rate limiting
        assert_struct!(response, ApiResponse {
            meta: ResponseMeta {
                rate_limit: RateLimit {
                    remaining: <= 100,                         // Within bounds
                    total: == 100,                             // Expected total
                    reset_at: =~ r"^\d{4}-\d{2}-\d{2}T.*Z$",  // Valid timestamp
                    
                    // Validate relationship between remaining and total
                    ..  
                },
                ..
            },
            ..
        });

        // Additional validation: ensure remaining is reasonable
        let rate_limit = &response.meta.rate_limit;
        assert!(rate_limit.remaining <= rate_limit.total, 
                "Remaining rate limit should not exceed total");
    }

    /// Test subscription feature validation
    #[test]
    fn test_premium_subscription_features() {
        let response = mock_successful_user_response();

        if let Some(ref user_data) = response.data {
            // Validate premium subscription has expected features
            assert_struct!(user_data, UserProfile {
                subscription: SubscriptionInfo {
                    tier: "premium",
                    
                    // Premium users should have these features
                    features: [
                        =~ r".*analytics.*",                   // Some analytics feature
                        =~ r".*support.*",                     // Some support feature
                        ..                                     // May have additional features
                    ],
                    
                    // Premium should have billing info
                    billing: BillingInfo {
                        amount: > 0.0,                         // Paid subscription
                        currency: != "",                       // Has currency
                        interval: =~ r"^(monthly|yearly)$",    // Valid billing cycle
                        next_billing_date: Some(_),            // Has next billing
                    },
                    
                    expires_at: Some(_),                       // Premium expires
                },
                ..
            });
        }
    }

    /// Test user engagement metrics
    #[test]
    fn test_user_engagement_patterns() {
        let response = mock_successful_user_response();

        if let Some(ref user_data) = response.data {
            assert_struct!(user_data, UserProfile {
                stats: UserStats {
                    // Engagement validation patterns
                    posts_count: > 0,                          // Active user
                    followers_count: > posts_count,            // Popular content
                    likes_received: > posts_count * 10,        // Good engagement ratio
                    join_date_days_ago: > 30,                  // Established account
                },
                
                // Verified users tend to be more engaged
                is_verified: true,
                
                ..
            });

            // Calculate engagement ratio (likes per post)
            let stats = &user_data.stats;
            if stats.posts_count > 0 {
                let engagement_ratio = stats.likes_received as f64 / stats.posts_count as f64;
                assert!(engagement_ratio > 50.0, 
                        "High engagement users should have good like-to-post ratio");
            }
        }
    }

    /// Test privacy settings combinations
    #[test]
    fn test_privacy_settings_consistency() {
        let response = mock_successful_user_response();

        if let Some(ref user_data) = response.data {
            assert_struct!(user_data, UserProfile {
                preferences: UserPreferences {
                    privacy: PrivacySettings {
                        profile_visibility: =~ r"^(public|private|friends)$",
                        ..
                    },
                    ..
                },
                ..
            });

            // Business logic: if profile is private, search indexing should be false
            let privacy = &user_data.preferences.privacy;
            if privacy.profile_visibility == "private" {
                assert_eq!(privacy.search_indexing, false, 
                          "Private profiles should not be indexed");
            }
        }
    }

    /// Test notification preferences validation
    #[test]
    fn test_notification_preferences() {
        let response = mock_successful_user_response();

        if let Some(ref user_data) = response.data {
            assert_struct!(user_data, UserProfile {
                preferences: UserPreferences {
                    notifications: NotificationSettings {
                        frequency: =~ r"^(immediate|daily|weekly)$",    // Valid frequency
                        
                        // At least one notification type should be enabled for active users
                        ..
                    },
                    ..
                },
                ..
            });

            // Business rule validation
            let notifications = &user_data.preferences.notifications;
            let has_any_notifications = notifications.email_notifications 
                || notifications.push_notifications;
            
            assert!(has_any_notifications, 
                    "Users should have at least one notification method enabled");
        }
    }
}

/// Integration test example showing how to test API client integration
#[cfg(test)]
mod integration_tests {
    use super::*;

    // Mock API client for demonstration
    struct ApiClient;

    impl ApiClient {
        async fn get_user_profile(&self, user_id: u64) -> Result<ApiResponse<UserProfile>, Box<dyn std::error::Error>> {
            // In real implementation, this would make HTTP request
            // For demo, return mock data based on user_id
            match user_id {
                12345 => Ok(mock_successful_user_response()),
                99999 => Ok(mock_error_response()),
                _ => panic!("Unexpected user_id in mock"),
            }
        }
    }

    #[tokio::test]
    async fn test_api_client_successful_response() {
        let client = ApiClient;
        let response = client.get_user_profile(12345).await.unwrap();

        // High-level API contract validation
        assert_struct!(response, ApiResponse {
            success: true,
            data: Some(_),                                     // Has user data
            error: None,                                       // No error
            meta: ResponseMeta {
                request_id.len(): > 0,                         // Valid request tracking
                version: =~ r"^v\d+\.\d+\.\d+$",              // API version format
                ..
            },
        });

        // Validate the actual user data structure
        if let Some(ref user_data) = response.data {
            assert_struct!(user_data, UserProfile {
                id: 12345,                                     // Expected user
                username.len(): > 3,                           // Reasonable username
                email.contains("@"): true,                     // Valid email format
                subscription: SubscriptionInfo {
                    tier: =~ r"^(free|premium|enterprise)$",   // Valid subscription
                    ..
                },
                ..
            });
        }
    }

    #[tokio::test]
    async fn test_api_client_error_handling() {
        let client = ApiClient;
        let response = client.get_user_profile(99999).await.unwrap();

        // Error response structure validation
        assert_struct!(response, ApiResponse::<UserProfile> {
            success: false,
            data: None,
            error: Some(ApiError {
                code: =~ r"^[A-Z_]+$",                        // Standard error code format
                message.is_empty(): false,                     // Has error message
                ..
            }),
            meta: ResponseMeta {
                request_id.starts_with("req_"): true,         // Request tracking works
                ..
            },
        });
    }
}

fn main() {
    println!("JSON API Validation Example");
    println!("Run with: cargo test --example json_api_validation");
}