use assert_struct::assert_struct;

#[derive(Debug)]
struct AsyncStruct {
    async_field: AsyncValue,
    regular_field: String,
}

#[derive(Debug)]
struct AsyncValue {
    value: i32,
}

impl AsyncValue {
    async fn get_value(&self) -> i32 {
        self.value
    }

    async fn get_doubled(&self) -> i32 {
        self.value * 2
    }
}

#[tokio::test]
async fn test_await_in_field_pattern() {
    let data = AsyncStruct {
        async_field: AsyncValue { value: 42 },
        regular_field: "test".to_string(),
    };

    // Test basic await pattern
    assert_struct!(data, AsyncStruct {
        async_field.get_value().await: 42,
        regular_field: "test",
        ..
    });
}

#[tokio::test]
async fn test_await_with_result_pattern() {
    let data = AsyncStruct {
        async_field: AsyncValue { value: 42 },
        regular_field: "test".to_string(),
    };

    // Test await with Result pattern
    assert_struct!(data, AsyncStruct {
        async_field.get_value().await: 42,
        ..
    });
}

#[tokio::test]
async fn test_await_with_comparison() {
    let data = AsyncStruct {
        async_field: AsyncValue { value: 50 },
        regular_field: "test".to_string(),
    };

    // Test await with comparison patterns
    assert_struct!(data, AsyncStruct {
        async_field.get_value().await: > 30,
        async_field.get_doubled().await: >= 100,
        ..
    });
}

#[tokio::test]
async fn test_tuple_with_await() {
    let data = (
        AsyncValue { value: 10 },
        AsyncValue { value: 20 },
        "regular".to_string(),
    );

    // Test await in tuple patterns
    assert_struct!(data, (
        0.get_value().await: 10,
        1.get_doubled().await: 40,
        "regular"
    ));
}

#[tokio::test]
async fn test_complex_nested_await() {
    #[derive(Debug)]
    struct NestedStruct {
        inner: AsyncStruct,
        counter: i32,
    }

    let data = NestedStruct {
        inner: AsyncStruct {
            async_field: AsyncValue { value: 100 },
            regular_field: "nested".to_string(),
        },
        counter: 5,
    };

    // Test nested structure with await
    assert_struct!(data, NestedStruct {
        inner: AsyncStruct {
            async_field.get_value().await: 100,
            regular_field: "nested",
            ..
        },
        counter: 5,
        ..
    });
}

#[tokio::test]
async fn test_wildcard_struct_with_await() {
    let data = AsyncStruct {
        async_field: AsyncValue { value: 75 },
        regular_field: "wildcard".to_string(),
    };

    // Test wildcard pattern with await
    assert_struct!(data, _ {
        async_field.get_value().await: 75,
        ..
    });
}

#[tokio::test]
async fn test_await_with_range() {
    let data = AsyncStruct {
        async_field: AsyncValue { value: 25 },
        regular_field: "range".to_string(),
    };

    // Test await with range patterns
    assert_struct!(data, AsyncStruct {
        async_field.get_value().await: 20..=30,
        async_field.get_doubled().await: 40..60,
        ..
    });
}
