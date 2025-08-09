use assert_struct::assert_struct;

#[derive(Debug, PartialEq)]
struct Point {
    x: i32,
    y: i32,
}

#[derive(Debug)]
struct Shape {
    origin: Point,
}

fn main() {
    let shape = Shape {
        origin: Point { x: 0, y: 0 },
    };

    // This should NOT compile - .. is not valid in expressions
    // The `..` rest pattern is only valid in pattern position, not in expressions
    assert_struct!(shape, Shape {
        origin: == Point { .. },
    });
}