#![allow(dead_code)]
use assert_struct::assert_struct;

#[derive(Debug)]
struct Settings {
    debug: Option<bool>,
    max_connections: Option<u32>,
}

#[derive(Debug)]
struct Environment {
    name: String,
    settings: Settings,
}

#[derive(Debug)]
struct Deployment {
    version: String,
    env: Environment,
}

pub fn test_case() {
    let deploy = Deployment {
        version: "2.0.0".to_string(),
        env: Environment {
            name: "staging".to_string(),
            settings: Settings {
                debug: Some(false),
                max_connections: Some(50),
            },
        },
    };

    assert_struct!(deploy, Deployment {
        version: "2.0.0",
        env: Environment {
            name: "staging",
            settings: Settings {
                debug: Some(true),  // Line 38 - should report this line (Some(false) != Some(true))
                max_connections: Some(50),
            },
        },
    });
}