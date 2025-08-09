//! Internal regex caching for performance optimization
//!
//! This module provides a thread-safe cache for compiled regex patterns
//! to avoid recompiling the same patterns repeatedly.

use regex::Regex;
use std::collections::HashMap;
use std::sync::RwLock;

// Global regex cache - thread-safe
lazy_static::lazy_static! {
    static ref REGEX_CACHE: RwLock<HashMap<String, Regex>> = RwLock::new(HashMap::new());
}

/// Get or compile a regex pattern from the cache
///
/// This function maintains a global cache of compiled regex patterns.
/// If the pattern exists in the cache, it returns a clone. Otherwise,
/// it compiles the pattern, stores it in the cache, and returns it.
pub(crate) fn get_or_compile_regex(pattern: &str) -> Option<Regex> {
    // First try to read from cache
    {
        let cache = REGEX_CACHE.read().ok()?;
        if let Some(regex) = cache.get(pattern) {
            return Some(regex.clone());
        }
    }
    
    // Not in cache, need to compile and store
    if let Ok(regex) = Regex::new(pattern) {
        if let Ok(mut cache) = REGEX_CACHE.write() {
            // Check again in case another thread added it while we were waiting
            if !cache.contains_key(pattern) {
                cache.insert(pattern.to_string(), regex.clone());
            }
        }
        Some(regex)
    } else {
        None
    }
}

/// Clear the regex cache (mainly useful for testing)
#[allow(dead_code)]
pub(crate) fn clear_regex_cache() {
    if let Ok(mut cache) = REGEX_CACHE.write() {
        cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_regex_cache() {
        // Clear cache to start fresh
        clear_regex_cache();
        
        // First call should compile and cache
        let pattern = r"test\d+";
        let regex1 = get_or_compile_regex(pattern).unwrap();
        
        // Second call should retrieve from cache
        let regex2 = get_or_compile_regex(pattern).unwrap();
        
        // Both should match the same things
        assert!(regex1.is_match("test123"));
        assert!(regex2.is_match("test456"));
        
        // Invalid regex should return None
        assert!(get_or_compile_regex(r"[").is_none());
    }
    
    #[test]
    fn test_cache_multiple_patterns() {
        clear_regex_cache();
        
        let patterns = vec![
            r"^hello",
            r"world$",
            r"\d{3}-\d{3}-\d{4}",
        ];
        
        // Cache all patterns
        for pattern in &patterns {
            get_or_compile_regex(pattern).unwrap();
        }
        
        // All should be cached now
        for pattern in &patterns {
            let regex = get_or_compile_regex(pattern).unwrap();
            assert!(regex.as_str() == *pattern);
        }
    }
}