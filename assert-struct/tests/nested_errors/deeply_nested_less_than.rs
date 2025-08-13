use assert_struct::assert_struct;

#[derive(Debug)]
struct Metrics {
    cpu_usage: f32,
    memory_mb: u32,
}

#[derive(Debug)]
struct Container {
    id: String,
    metrics: Metrics,
}

#[derive(Debug)]
struct Pod {
    name: String,
    container: Container,
}

pub fn test_case() {
    let pod = Pod {
        name: "web-pod".to_string(),
        container: Container {
            id: "abc123".to_string(),
            metrics: Metrics {
                cpu_usage: 75.5,
                memory_mb: 1024,
            },
        },
    };

    assert_struct!(pod, Pod {
        name: "web-pod",
        container: Container {
            id: "abc123",
            metrics: Metrics {
                cpu_usage: < 50.0,  // Line 38 - should report this line (75.5 is not < 50.0)
                memory_mb: 1024,
            },
        },
    });
}