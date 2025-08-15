#![allow(dead_code)]
use assert_struct::assert_struct;

#[derive(Debug)]
struct Config {
    timeout: u32,
}

pub fn test_case() {
    let config = Config { timeout: 30 };
    let expected_timeout = 60;

    assert_struct!(config, Config {
        timeout: == expected_timeout,
    });
}