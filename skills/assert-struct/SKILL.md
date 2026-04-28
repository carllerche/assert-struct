---
name: assert-struct-guidance
description: Always use this skill before writing any test code in the Toasty repository
crates: assert-struct
activation: always
---

The `assert-struct` crate helps to write concise assertions for the values of struct fields.

# assert_struct! rule

> Use whichever form produces **fewer characters** and the **same test coverage**.

```rust
// assert_eq! wins — single field
assert_eq!(foo.name, "hello");

// assert_struct! wins — multiple fields in one call
assert_struct!(foo, _ { name: "hello", age: 30 });
// vs. assert_eq!(foo.name, "hello"); assert_eq!(foo.age, 30);

// assert_eq! wins — you have the whole value
assert_eq!(result, Foo::default());
```

# assert_struct! quick reference

Patterns compose freely. Use `_` for wildcard struct (no import needed), `..` for partial match:

```rust
assert_struct!(val, _ { field: "text", count: > 0, flag: true, .. });
assert_struct!(val, _ { opt: Some(42), res: Ok("ok"), .. });
assert_struct!(val, _ { items: [1, 2, ..], tags: #("a", "b", ..), .. });
assert_struct!(val, _ { nested.child.x: >= 0, .. });  // dot-path shorthand
assert_struct!(val, _ { items.len(): 3, .. });         // method call
```

Operator patterns at leaves avoid importing types:
```rust
// Instead of:  field: SomeEnum::Variant(42)
// Write:        field: == expected_var
```

The full pattern grammar can be found in the `LLM.txt` resource within this skill.

# assert_struct! formatting

**rustfmt does not format `assert_struct!` bodies.** You are responsible for formatting them manually, using the same rules rustfmt would apply to equivalent Rust code.

**One-liner:** If the entire call fits on one line within the column limit (~100 chars), keep it on one line.

```rust
assert_struct!(resp, _ { rows: Rows::Count(1), .. });
```

**Multi-line:** When breaking across lines, treat the macro body like a struct literal or match arm — indent the contents by 4 spaces relative to the `assert_struct!` call, and put the closing `});` on its own line.

```rust
assert_struct!(op, Operation::QuerySql(_ {
    stmt: Statement::Update(_ {
        target: UpdateTarget::Table(== foo_table_id),
        assignments: #{ 1: _ { expr: == "new", .. } },
        ..
    }),
    ..
}));
```

**When the macro syntax differs from Rust** (e.g. `#{ .. }` for map patterns, `== expr` leaf operators, dot-path keys like `items.len():`), use your best judgment to apply equivalent rustfmt rules: align commas consistently, indent nested levels, and never leave trailing whitespace.
