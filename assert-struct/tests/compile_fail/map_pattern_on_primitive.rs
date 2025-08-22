use assert_struct::assert_struct;

#[derive(Debug)]
struct Config {
    port: u16,
}

fn main() {
    let config = Config {
        port: 8080,
    };

    // This should NOT compile - u16 doesn't have len() or get() methods
    assert_struct!(config, Config {
        port: #{ "key": 443 },
    });
}