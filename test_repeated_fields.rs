use assert_struct::assert_struct;

#[derive(Debug)]
struct User {
    name: String,
    age: u32,
}

#[derive(Debug)]  
struct Container {
    foo: InnerStruct,
}

#[derive(Debug)]
struct InnerStruct {
    bar: i32,
    baz: i32,
}

fn main() {
    let user = User {
        name: "Alice".to_string(),
        age: 25,
    };

    // Test case 1: Repeated field patterns
    println!("Testing repeated field patterns...");
    assert_struct!(user, User { 
        age: >= 10, 
        age: <= 99,
        .. 
    });
    println!("✓ Repeated field patterns work!");

    // Test case 2: Repeated method/field access patterns  
    let container = Container {
        foo: InnerStruct { bar: 1, baz: 2 },
    };
    
    println!("Testing repeated method/field access patterns...");
    assert_struct!(container, Container { 
        foo.bar: 1, 
        foo.baz: 2,
        ..
    });
    println!("✓ Repeated method/field access patterns work!");
    
    println!("All tests passed!");
}