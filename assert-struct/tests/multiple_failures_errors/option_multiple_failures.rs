#![allow(dead_code)]
use assert_struct::assert_struct;

#[derive(Debug)]
struct Settings {
    theme: Option<String>,
    font_size: Option<u32>,
    auto_save: Option<bool>,
}

pub fn test_case() {
    let settings = Settings {
        theme: Some("light".to_string()),  // Will fail: expected "dark"
        font_size: Some(10),  // Will fail: expected > 12
        auto_save: Some(true),
    };

    assert_struct!(settings, Settings {
        theme: Some("dark"),
        font_size: Some(> 12),
        auto_save: Some(true),
    });
}