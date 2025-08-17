// Test current behavior with Box fields and plan for auto-deref implementation

use assert_struct::assert_struct;
use std::rc::Rc;
use std::sync::Arc;

#[derive(Debug)]
struct TestStruct {
    boxed_value: Box<Option<i32>>,
    rc_value: Rc<String>,
    arc_value: Arc<u32>,
    normal_value: i32,
}

#[test]
fn test_current_box_behavior() {
    let test = TestStruct {
        boxed_value: Box::new(Some(42)),
        rc_value: Rc::new("hello".to_string()),
        arc_value: Arc::new(100),
        normal_value: 200,
    };

    // This currently works - normal value
    assert_struct!(test, TestStruct {
        normal_value: 200,
        ..
    });

    // This currently requires closures for Box/Rc/Arc
    assert_struct!(test, TestStruct {
        boxed_value: |b| matches!(**b, Some(42)),
        rc_value: |s| s.as_str() == "hello",
        arc_value: |a| **a == 100,
        normal_value: 200,
        ..
    });
}

// TODO: These tests will be enabled once auto-deref is implemented
// 
// #[test]
// fn test_auto_deref_goal() {
//     let test = TestStruct {
//         boxed_value: Box::new(Some(42)),
//         rc_value: Rc::new("hello".to_string()),
//         arc_value: Arc::new(100),
//         normal_value: 200,
//     };
//
//     // This is what we want to achieve with auto-deref:
//     assert_struct!(test, TestStruct {
//         boxed_value: Some(42),      // Auto-deref Box<Option<i32>> -> Option<i32>
//         rc_value: "hello",          // Auto-deref Rc<String> -> String
//         arc_value: 100,             // Auto-deref Arc<u32> -> u32
//         normal_value: 200,          // No deref needed
//         ..
//     });
// }
//
// #[test]
// fn test_nested_smart_pointers_goal() {
//     #[derive(Debug)]
//     struct NestedStruct {
//         complex: Box<Rc<Option<String>>>,
//     }
//
//     let nested = NestedStruct {
//         complex: Box::new(Rc::new(Some("test".to_string()))),
//     };
//
//     // Goal: Handle nested smart pointers automatically
//     assert_struct!(nested, NestedStruct {
//         complex: Some("test"),  // Auto-deref Box<Rc<Option<String>>> -> Option<String>
//     });
// }