#![allow(dead_code)]
use assert_struct::assert_struct;

#[derive(Debug)]
struct Coordinates {
    position: (f64, f64, f64),
    rotation: (f32, f32),
}

#[derive(Debug)]
struct Transform {
    name: String,
    coords: Coordinates,
}

#[derive(Debug)]
struct GameObject {
    id: u32,
    transform: Transform,
}

pub fn test_case() {
    let obj = GameObject {
        id: 42,
        transform: Transform {
            name: "player".to_string(),
            coords: Coordinates {
                position: (10.0, 20.0, 30.0),
                rotation: (0.0, 90.0),
            },
        },
    };

    assert_struct!(obj, GameObject {
        id: 42,
        transform: Transform {
            name: "player",
            coords: Coordinates {
                position: (10.0, 25.0, 30.0),  // Line 38 - should report this line (20.0 != 25.0)
                rotation: (0.0, 90.0),
            },
        },
    });
}