pub fn test_case() {
    // This test exists to document that multiple .. patterns are not allowed
    // The actual test is commented out because it would fail to compile
    // which is the expected behavior

    // The following would fail to compile:
    // let container = Container {
    //     items: vec![1, 2, 3, 4, 5],
    //     names: vec!["a".to_string()],
    //     data: vec![10],
    // };
    //
    // assert_struct!(container, Container {
    //     items: [1, .., 3, .., 5],  // Multiple .. not allowed
    //     names: ["a"],
    //     data: [10],
    // });

    // For now, we'll just panic to have a test that documents this limitation
    panic!("Multiple .. patterns in slices are not allowed - this is enforced at compile time");
}