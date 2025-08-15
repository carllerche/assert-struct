#![allow(dead_code)]
use assert_struct::assert_struct;

#[derive(Debug)]
struct Database {
    version: u32,
    connections: i32,
}

#[derive(Debug)]
struct Server {
    port: u16,
    database: Database,
}

#[derive(Debug)]
struct System {
    name: String,
    server: Server,
}

pub fn test_case() {
    let system = System {
        name: "prod".to_string(),
        server: Server {
            port: 8080,
            database: Database {
                version: 5,
                connections: 10,
            },
        },
    };

    assert_struct!(system, System {
        name: "prod",
        server: Server {
            port: 8080,
            database: Database {
                version: == 6,  // Line 38 - should report this line
                connections: 10,
            },
        },
    });
}