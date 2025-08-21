use assert_struct::assert_struct;

#[derive(Debug)]
struct Container {
    value: String,
}

fn main() {
    let container = Container { 
        value: "test".to_string() 
    };
    
    // This should fail with a type error pointing to the specific closure parameter
    assert_struct!(container, Container {
        value: |x: i32| x > 0,  // Type error: closure expects i32 but gets &String
        ..
    });
}