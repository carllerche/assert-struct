// Start with a simple auto-deref test case
use assert_struct::assert_struct;

#[derive(Debug)]
struct SimpleBox {
    boxed_option: Box<Option<i32>>,
}

#[test]
fn test_box_option_with_closure() {
    let test = SimpleBox {
        boxed_option: Box::new(Some(42)),
    };

    // This currently works with closures
    assert_struct!(test, SimpleBox {
        boxed_option: |b| matches!(**b, Some(42)),
    });
}

// TODO: Enable once auto-deref is implemented
// #[test]
// fn test_box_option_auto_deref() {
//     let test = SimpleBox {
//         boxed_option: Box::new(Some(42)),
//     };
//
//     // Test auto-deref: Box<Option<i32>> -> Option<i32>
//     assert_struct!(test, SimpleBox {
//         boxed_option: Some(42),  // Should auto-deref Box<Option<i32>> -> Option<i32>
//     });
// }