#![allow(dead_code)]
use assert_struct::assert_struct;

#[derive(Debug)]
struct Config {
    timeout: u32,
    retries: u8,
}

#[derive(Debug)]
struct Service {
    enabled: bool,
    config: Config,
}

#[derive(Debug)]
struct Application {
    version: String,
    service: Service,
}

pub fn test_case() {
    let app = Application {
        version: "1.0.0".to_string(),
        service: Service {
            enabled: true,
            config: Config {
                timeout: 30,
                retries: 3,
            },
        },
    };

    assert_struct!(app, Application {
        version: "1.0.0",
        service: Service {
            enabled: true,
            config: Config {
                timeout: != 30,  // Line 38 - should report this line (will fail because it IS 30)
                retries: 3,
            },
        },
    });
}