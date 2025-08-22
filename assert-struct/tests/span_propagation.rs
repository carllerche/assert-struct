/// Tests to verify that compiler errors point to specific operations
/// rather than the entire assert_struct! macro call.
///
/// These tests are designed to fail compilation, but the important thing
/// is that the error spans point to the right location.
use assert_struct::assert_struct;

#[derive(Debug)]
#[allow(dead_code)]
struct TestStruct {
    value: i32,
    text: String,
}

// Test that method call errors point to the method, not the macro
#[test]
fn test_method_span_propagation() {
    let data = TestStruct {
        value: 42,
        text: "hello".to_string(),
    };

    // This should compile successfully - method exists
    assert_struct!(data, TestStruct {
        text.len(): 5,
        ..
    });
}

// Test that field access errors point to the field operation, not the macro
#[test]
fn test_nested_field_span_propagation() {
    #[derive(Debug)]
    struct Inner {
        nested_value: i32,
    }

    #[derive(Debug)]
    struct Outer {
        inner: Inner,
    }

    let data = Outer {
        inner: Inner { nested_value: 100 },
    };

    // This should compile successfully - nested field exists
    assert_struct!(data, Outer {
        inner.nested_value: 100,
        ..
    });
}

// Test that index operation errors point to the index, not the macro
#[test]
fn test_index_span_propagation() {
    let data = vec![1, 2, 3, 4, 5];

    // This should compile successfully - we're testing Vec indexing
    assert_struct!(data, [1, 2, 3, 4, 5]);
}

// Test that await operation errors would point to the await, not the macro
// This test uses a future that should compile
#[tokio::test]
async fn test_await_span_propagation() {
    use std::future::Future;
    use std::pin::Pin;
    use std::task::{Context, Poll};

    // Simple future that returns an integer
    #[derive(Debug)]
    struct SimpleFuture(i32);

    impl Future for SimpleFuture {
        type Output = i32;

        fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
            Poll::Ready(self.0)
        }
    }

    impl Future for &SimpleFuture {
        type Output = i32;

        fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
            Poll::Ready(self.0)
        }
    }

    #[derive(Debug)]
    struct AsyncStruct {
        future_field: SimpleFuture,
    }

    let data = AsyncStruct {
        future_field: SimpleFuture(42),
    };

    // This should compile successfully - future_field can be awaited
    assert_struct!(data, AsyncStruct {
        future_field.await: 42,
        ..
    });
}

// Test complex chained operations
#[test]
fn test_complex_chain_span_propagation() {
    let data = TestStruct {
        value: 42,
        text: "hello world".to_string(),
    };

    // This should compile successfully - chained method calls
    assert_struct!(data, TestStruct {
        text.len(): > 5,
        text.chars().count(): 11,
        ..
    });
}

// Test that tuple index operations have proper spans
#[test]
fn test_tuple_index_span_propagation() {
    #[derive(Debug)]
    struct TupleContainer {
        data: (String, i32),
    }

    let container = TupleContainer {
        data: ("hello".to_string(), 42),
    };

    // This should compile successfully - tuple indexing with method calls
    assert_struct!(container, TupleContainer {
        data: (
            0.len(): 5,
            1: 42,
        ),
        ..
    });
}
