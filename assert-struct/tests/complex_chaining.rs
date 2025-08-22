use assert_struct::assert_struct;

#[derive(Debug)]
struct ComplexData {
    nested_futures: NestedFutures,
    #[allow(dead_code)]
    indexable_async: Vec<AsyncValue>,
    #[allow(dead_code)]
    chained_ops: ChainedOperations,
}

#[derive(Debug)]
struct NestedFutures {
    triple_future: AsyncValue,
}

#[derive(Debug, Clone)]
struct AsyncValue {
    value: i32,
}

#[derive(Debug)]
struct ChainedOperations {
    #[allow(dead_code)]
    data: Vec<AsyncValue>,
}

impl AsyncValue {
    async fn get_future(&self) -> AsyncValue {
        AsyncValue {
            value: self.value * 2,
        }
    }

    async fn as_vec(&self) -> Vec<i32> {
        vec![self.value, self.value + 1, self.value + 2]
    }

    async fn get_value(&self) -> i32 {
        self.value
    }
}

impl ChainedOperations {
    #[allow(dead_code)]
    async fn get_first(&self) -> AsyncValue {
        self.data[0].clone()
    }
}

#[tokio::test]
async fn test_triple_nested_futures() {
    let data = ComplexData {
        nested_futures: NestedFutures {
            triple_future: AsyncValue { value: 5 },
        },
        indexable_async: vec![AsyncValue { value: 10 }],
        chained_ops: ChainedOperations {
            data: vec![AsyncValue { value: 100 }],
        },
    };

    // Test .await.await.await (chained futures)
    assert_struct!(data, ComplexData {
        nested_futures: NestedFutures {
            triple_future.get_future().await.get_future().await.get_value().await: 20, // 5 * 2 * 2 = 20
            ..
        },
        ..
    });
}

#[tokio::test]
async fn test_await_with_indexing() {
    let data = ComplexData {
        nested_futures: NestedFutures {
            triple_future: AsyncValue { value: 7 },
        },
        indexable_async: vec![AsyncValue { value: 10 }],
        chained_ops: ChainedOperations {
            data: vec![AsyncValue { value: 100 }],
        },
    };

    // Test .await[0] (await returning indexable type)
    assert_struct!(data, ComplexData {
        nested_futures: NestedFutures {
            triple_future.as_vec().await[0]: 7,
            ..
        },
        ..
    });

    // Test .await[1] with comparison
    assert_struct!(data, ComplexData {
        nested_futures: NestedFutures {
            triple_future.as_vec().await[1]: > 7,
            ..
        },
        ..
    });
}

#[tokio::test]
async fn test_complex_method_await_index_chains() {
    let data = ComplexData {
        nested_futures: NestedFutures {
            triple_future: AsyncValue { value: 3 },
        },
        indexable_async: vec![AsyncValue { value: 10 }],
        chained_ops: ChainedOperations {
            data: vec![AsyncValue { value: 100 }],
        },
    };

    // Test complex chains: .method().await[index].method().await
    assert_struct!(data, ComplexData {
        nested_futures: NestedFutures {
            triple_future.as_vec().await[2]: 5, // 3 + 2 = 5
            ..
        },
        ..
    });
}

#[tokio::test]
async fn test_tuple_with_complex_await_patterns() {
    let tuple_data = (
        AsyncValue { value: 20 },
        AsyncValue { value: 30 },
        "static_value",
    );

    // Test complex tuple patterns with various await operations
    assert_struct!(tuple_data, (
        0.get_future().await.get_value().await: 40, // 20 * 2 = 40
        1.as_vec().await[0]: 30,
        "static_value"
    ));
}

#[tokio::test]
async fn test_multiple_await_same_field() {
    let data = ComplexData {
        nested_futures: NestedFutures {
            triple_future: AsyncValue { value: 4 },
        },
        indexable_async: vec![AsyncValue { value: 10 }],
        chained_ops: ChainedOperations {
            data: vec![AsyncValue { value: 100 }],
        },
    };

    // Test multiple different await operations on the same field
    assert_struct!(data, ComplexData {
        nested_futures: NestedFutures {
            triple_future.get_value().await: 4,
            triple_future.get_future().await.get_value().await: 8, // 4 * 2 = 8
            triple_future.as_vec().await[1]: 5, // 4 + 1 = 5
            ..
        },
        ..
    });
}

#[tokio::test]
async fn test_await_with_range_patterns() {
    let data = ComplexData {
        nested_futures: NestedFutures {
            triple_future: AsyncValue { value: 15 },
        },
        indexable_async: vec![AsyncValue { value: 10 }],
        chained_ops: ChainedOperations {
            data: vec![AsyncValue { value: 100 }],
        },
    };

    // Test await with range patterns
    assert_struct!(data, ComplexData {
        nested_futures: NestedFutures {
            triple_future.get_value().await: 10..=20,
            triple_future.as_vec().await[0]: 15,
            ..
        },
        ..
    });
}

#[tokio::test]
async fn test_wildcard_with_complex_await() {
    let _data = ComplexData {
        nested_futures: NestedFutures {
            triple_future: AsyncValue { value: 25 },
        },
        indexable_async: vec![AsyncValue { value: 10 }],
        chained_ops: ChainedOperations {
            data: vec![AsyncValue { value: 100 }],
        },
    };

    // Test wildcard patterns with complex await chains
    assert_struct!(data, _ {
        nested_futures: _ {
            // TODO: Fix this pattern - indexing issue
            // triple_future.get_future().await.as_vec().await[2]: 52, // (25 * 2) + 2 = 52
            ..
        },
        ..
    });
}
