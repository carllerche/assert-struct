use assert_struct::assert_struct;

// Test 1: Large structures with pattern pruning
#[derive(Debug)]
#[allow(dead_code)]
struct LargeStruct {
    field01: String,
    field02: u32,
    field03: String,
    field04: u32,
    field05: String,
    field06: u32,
    field07: String,
    field08: u32,
    field09: String,
    field10: u32,
    field11: String,
    field12: u32,
    field13: String,
    field14: u32,
    field15: String,
    field16: u32,
    field17: String,
    field18: u32,
    field19: String,
    field20: u32,
    field21: String,
    field22: u32,
    field23: String,
    field24: u32,
    field25: String,
    field26: u32,
    field27: String,
    field28: u32,
    field29: String,
    field30: u32,
    important_field: u32, // This is the field that will fail
    field31: String,
    field32: u32,
    field33: String,
    field34: u32,
    field35: String,
    field36: u32,
    field37: String,
    field38: u32,
    field39: String,
    field40: u32,
    field41: String,
    field42: u32,
    field43: String,
    field44: u32,
    field45: String,
    field46: u32,
    field47: String,
    field48: u32,
    field49: String,
    field50: u32,
}

#[test]
#[should_panic(expected = "important_field")]
fn test_large_struct_pruning() {
    let large = LargeStruct {
        field01: "a".to_string(),
        field02: 1,
        field03: "b".to_string(),
        field04: 2,
        field05: "c".to_string(),
        field06: 3,
        field07: "d".to_string(),
        field08: 4,
        field09: "e".to_string(),
        field10: 5,
        field11: "f".to_string(),
        field12: 6,
        field13: "g".to_string(),
        field14: 7,
        field15: "h".to_string(),
        field16: 8,
        field17: "i".to_string(),
        field18: 9,
        field19: "j".to_string(),
        field20: 10,
        field21: "k".to_string(),
        field22: 11,
        field23: "l".to_string(),
        field24: 12,
        field25: "m".to_string(),
        field26: 13,
        field27: "n".to_string(),
        field28: 14,
        field29: "o".to_string(),
        field30: 15,
        important_field: 100,
        field31: "p".to_string(),
        field32: 16,
        field33: "q".to_string(),
        field34: 17,
        field35: "r".to_string(),
        field36: 18,
        field37: "s".to_string(),
        field38: 19,
        field39: "t".to_string(),
        field40: 20,
        field41: "u".to_string(),
        field42: 21,
        field43: "v".to_string(),
        field44: 22,
        field45: "w".to_string(),
        field46: 23,
        field47: "x".to_string(),
        field48: 24,
        field49: "y".to_string(),
        field50: 25,
    };

    // This should fail and show context around important_field with pruning
    assert_struct!(large, LargeStruct {
        important_field: > 200,  // Fails: 100 is not > 200
        ..
    });
}

// Test 2: Complex nested slices (slices containing structs with more slices)
#[derive(Debug)]
struct Item {
    id: u32,
    tags: Vec<String>,
    scores: Vec<i32>,
}

#[derive(Debug)]
struct Inventory {
    items: Vec<Item>,
}

#[test]
fn test_nested_slice_structs() {
    let inventory = Inventory {
        items: vec![
            Item {
                id: 1,
                tags: vec!["red".to_string(), "large".to_string()],
                scores: vec![10, 20, 30],
            },
            Item {
                id: 2,
                tags: vec!["blue".to_string(), "small".to_string()],
                scores: vec![5, 15, 25],
            },
            Item {
                id: 3,
                tags: vec!["green".to_string()],
                scores: vec![100],
            },
        ],
    };

    // This complex pattern should work
    assert_struct!(inventory, Inventory {
        items: [
            Item {
                id: 1,
                tags: ["red", "large"],
                scores: [10, 20, 30],
            },
            Item {
                id: 2,
                tags: ["blue", "small"],
                scores: [> 0, < 20, == 25],
            },
            Item {
                id: 3,
                tags: ["green"],
                scores: [>= 100],
            },
        ],
    });
}

#[test]
#[should_panic(expected = "inventory.items.[1].scores.[1]")]
fn test_nested_slice_structs_failure() {
    let inventory = Inventory {
        items: vec![
            Item {
                id: 1,
                tags: vec!["red".to_string()],
                scores: vec![10],
            },
            Item {
                id: 2,
                tags: vec!["blue".to_string()],
                scores: vec![5, 15, 25], // 15 will fail the check
            },
        ],
    };

    assert_struct!(inventory, Inventory {
        items: [
            Item {
                id: 1,
                tags: ["red"],
                scores: [10],
            },
            Item {
                id: 2,
                tags: ["blue"],
                scores: [5, > 20, 25],  // Fails: 15 is not > 20
            },
        ],
    });
}

// Test 3: Very deep nesting
#[derive(Debug)]
#[allow(dead_code)]
struct Level1 {
    name: String,
    level2: Level2,
    extra_field1: String,
    extra_field2: u32,
    extra_field3: String,
}

#[derive(Debug)]
#[allow(dead_code)]
struct Level2 {
    name: String,
    level3: Level3,
    extra_field1: String,
    extra_field2: u32,
    extra_field3: String,
}

#[derive(Debug)]
#[allow(dead_code)]
struct Level3 {
    name: String,
    level4: Level4,
    extra_field1: String,
    extra_field2: u32,
    extra_field3: String,
}

#[derive(Debug)]
#[allow(dead_code)]
struct Level4 {
    name: String,
    level5: Level5,
    extra_field1: String,
    extra_field2: u32,
    extra_field3: String,
}

#[derive(Debug)]
#[allow(dead_code)]
struct Level5 {
    name: String,
    level6: Level6,
    extra_field1: String,
    extra_field2: u32,
    extra_field3: String,
}

#[derive(Debug)]
#[allow(dead_code)]
struct Level6 {
    name: String,
    deep_value: u32,
    extra_field1: String,
    extra_field2: u32,
    extra_field3: String,
}

#[test]
#[should_panic(expected = "deep.level2.level3.level4.level5.level6.deep_value")]
fn test_very_deep_nesting() {
    let deep = Level1 {
        name: "L1".to_string(),
        level2: Level2 {
            name: "L2".to_string(),
            level3: Level3 {
                name: "L3".to_string(),
                level4: Level4 {
                    name: "L4".to_string(),
                    level5: Level5 {
                        name: "L5".to_string(),
                        level6: Level6 {
                            name: "L6".to_string(),
                            deep_value: 42,
                            extra_field1: "x".to_string(),
                            extra_field2: 1,
                            extra_field3: "y".to_string(),
                        },
                        extra_field1: "x".to_string(),
                        extra_field2: 1,
                        extra_field3: "y".to_string(),
                    },
                    extra_field1: "x".to_string(),
                    extra_field2: 1,
                    extra_field3: "y".to_string(),
                },
                extra_field1: "x".to_string(),
                extra_field2: 1,
                extra_field3: "y".to_string(),
            },
            extra_field1: "x".to_string(),
            extra_field2: 1,
            extra_field3: "y".to_string(),
        },
        extra_field1: "x".to_string(),
        extra_field2: 1,
        extra_field3: "y".to_string(),
    };

    // This should fail deep in the structure
    assert_struct!(deep, Level1 {
        level2: Level2 {
            level3: Level3 {
                level4: Level4 {
                    level5: Level5 {
                        level6: Level6 {
                            deep_value: > 100,  // Fails: 42 is not > 100
                            ..
                        },
                        ..
                    },
                    ..
                },
                ..
            },
            ..
        },
        ..
    });
}

// Test 4: Multiple fields with the same name in nested structures
#[derive(Debug)]
struct Container {
    name: String,
    id: u32,
    inner: InnerContainer,
}

#[derive(Debug)]
struct InnerContainer {
    name: String, // Same field name as parent
    id: u32,      // Same field name as parent
    value: u32,
}

#[test]
fn test_same_field_names() {
    let container = Container {
        name: "outer".to_string(),
        id: 1,
        inner: InnerContainer {
            name: "inner".to_string(),
            id: 2,
            value: 100,
        },
    };

    // Test that we correctly match the right fields
    assert_struct!(container, Container {
        name: "outer",
        id: 1,
        inner: InnerContainer {
            name: "inner",
            id: 2,
            value: >= 100,
        },
    });
}

#[test]
#[should_panic(expected = "container.inner.name")]
fn test_same_field_names_inner_failure() {
    let container = Container {
        name: "outer".to_string(),
        id: 1,
        inner: InnerContainer {
            name: "wrong".to_string(),
            id: 2,
            value: 100,
        },
    };

    // Should fail on inner.name, not outer name
    assert_struct!(
        container,
        Container {
            name: "outer",
            id: 1,
            inner: InnerContainer {
                name: "inner", // Fails: "wrong" != "inner"
                id: 2,
                value: 100,
            },
        }
    );
}

#[test]
#[should_panic(expected = "container.name")]
fn test_same_field_names_outer_failure() {
    let container = Container {
        name: "wrong".to_string(),
        id: 1,
        inner: InnerContainer {
            name: "inner".to_string(),
            id: 2,
            value: 100,
        },
    };

    // Should fail on outer name, not inner.name
    assert_struct!(
        container,
        Container {
            name: "outer", // Fails: "wrong" != "outer"
            id: 1,
            inner: InnerContainer {
                name: "inner",
                id: 2,
                value: 100,
            },
        }
    );
}

// Test 5: Pattern with mixed complexity - some simple fields, some complex nested
#[derive(Debug)]
#[allow(dead_code)]
struct MixedComplexity {
    simple1: String,
    simple2: u32,
    complex: ComplexPart,
    simple3: String,
    simple4: u32,
}

#[derive(Debug)]
#[allow(dead_code)]
struct ComplexPart {
    items: Vec<SubItem>,
    metadata: Metadata,
}

#[derive(Debug)]
struct SubItem {
    id: u32,
    values: Vec<i32>,
}

#[derive(Debug)]
#[allow(dead_code)]
struct Metadata {
    created: String,
    updated: String,
    tags: Vec<String>,
}

#[test]
#[should_panic(expected = "mixed.complex.items.[1].values.[0]")]
fn test_mixed_complexity_deep_failure() {
    let mixed = MixedComplexity {
        simple1: "start".to_string(),
        simple2: 10,
        complex: ComplexPart {
            items: vec![
                SubItem {
                    id: 1,
                    values: vec![10, 20],
                },
                SubItem {
                    id: 2,
                    values: vec![5, 15], // 5 will fail the check
                },
            ],
            metadata: Metadata {
                created: "2024-01-01".to_string(),
                updated: "2024-01-02".to_string(),
                tags: vec!["tag1".to_string(), "tag2".to_string()],
            },
        },
        simple3: "end".to_string(),
        simple4: 20,
    };

    assert_struct!(mixed, MixedComplexity {
        complex: ComplexPart {
            items: [
                SubItem {
                    id: 1,
                    values: [10, 20],
                },
                SubItem {
                    id: 2,
                    values: [> 10, 15],  // Fails: 5 is not > 10
                },
            ],
            ..
        },
        ..
    });
}
