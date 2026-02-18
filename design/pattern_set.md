# Set Pattern

## Overview

The set pattern `#(...)` asserts that a collection contains items matching the given patterns, **in any order**. Unlike the slice pattern `[...]` which is position-sensitive, the set pattern treats the collection as an unordered bag.

Duck-typed: works on anything with `len()` and that implements `IntoIterator` (or is iterable via `&collection`).

## Syntax

```rust
// Exact: collection has exactly these 3 elements (in any order)
assert_struct!(foo, #(1, 2, 3));

// Rest: collection has at least these 2 elements, possibly more
assert_struct!(foo, #(1, 2, ..));

// Empty: collection is empty
assert_struct!(foo, #());

// Wildcard-only: collection has any contents (equivalent to just not asserting)
assert_struct!(foo, #(..));
```

## All Pattern Types Supported Inside

Any assert_struct! pattern is valid as a set element:

```rust
// Comparison operators
assert_struct!(scores, #(> 90, < 50, ..));

// Struct patterns (wildcard or named)
assert_struct!(events, #(
    _ { kind: "click", .. },
    _ { kind: "hover", .. },
    ..
));

// Enum patterns
assert_struct!(results, #(Some(> 0), None, ..));

// Ranges
assert_struct!(ages, #(18..=30, 31..=65));
```

## Semantics

Each pattern in `#(...)` must match a **distinct** element. Matching uses **backtracking**: for each pattern in order, try each unclaimed element; if the pattern matches, claim it and recurse to the next pattern; if the recursion fails, unclaim and try the next element. This guarantees a valid assignment is found whenever one exists, avoiding false failures from greedy competition between patterns.

```rust
// foo = [3, 1, 2]
assert_struct!(foo, #(1, 2, 3));   // passes
assert_struct!(foo, #(3, 1, 2));   // passes (order irrelevant)
assert_struct!(foo, #(1, 2));      // fails: exact count, foo has 3 elements
assert_struct!(foo, #(1, 2, ..));  // passes: at least 1 and 2 present
```

**Without `..`**: `len()` must equal the number of patterns.
**With `..`**: `len()` must be ≥ the number of non-`..` patterns.

## As a Field Value

```rust
assert_struct!(container, Container {
    tags: #("rust", "async", ..),
    scores: #(> 80, > 90),
    ..
});
```

## Error Messages

On failure, the error reports which patterns went unmatched:

```
assert_struct! failed:

  | Container { tags: #("rust", "async", ..) }
  |                         ^^^^^^^ no element matched this pattern
  | actual: ["rust", "python"]
```

## What This Is Not

- Not a `HashSet`/`BTreeSet` assertion — duck-typed, works on `Vec`, arrays, custom iterables
- Not a subset check — without `..`, the count must match exactly
- Not `#{ }` (map pattern) — `#(...)` uses parens for the unordered collection pattern

---

## Implementation

### Components (in implementation order)

- [ ] **`PatternSet` struct** — new file `assert-struct-macros/src/pattern/set.rs`. Holds `Vec<Pattern>` and `bool rest`. Implements `Parse`, `Display`, and carries a `node_id`.
- [ ] **`Pattern::Set` variant** — add to `pattern.rs` enum; update `span()`, `location()`, `Display`, and parsing dispatch (`peek(#) && peek2(Paren)` → `PatternSet`, distinct from `peek2(Brace)` → `Map`).
- [ ] **`ErrorReport::silence()` / `unsilence()`** — add to `assert-struct/src/error.rs`. A silenced report accepts `push()` calls but discards them. Used by predicate closures during backtracking to probe patterns without leaking trial failures into the real report.
- [ ] **`set_match` runtime helper** — add to `assert-struct/src/__macro_support`. Owns the length check, backtracking algorithm, and error reporting. Signature:
  ```rust
  pub fn set_match(
      n_elements: usize,
      rest: bool,
      predicates: &[&dyn Fn(usize) -> bool],
      report: &mut ErrorReport,
      node: &PatternNode,
  )
  ```
- [ ] **`NodeKind::Set`** — add to `assert-struct/src/error.rs` alongside `Slice` and `Map`; used for error display.
- [ ] **`expand_set_assertion`** — add to `assert-struct-macros/src/expand.rs`. Generates the minimal expansion (see below).
- [ ] **Pattern node generation** — add `Pattern::Set` arm to `expand/nodes.rs`, mirroring `Slice`.
- [ ] **Tests** — `assert-struct/tests/sets.rs`: exact match, rest pattern, complex element patterns, failure cases, error snapshot tests.

### Macro Expansion Design

The macro generates exactly three things; all other logic lives in the runtime:

```rust
// 1. Collect for indexed access (element type stays out of the runtime)
let __set_coll: Vec<_> = (&expr).into_iter().collect();

// 2. N predicate closures — one per pattern.
//    Each shadows __report with a silenced copy so existing assertion
//    code generation is reused unchanged.
let __preds: &[&dyn Fn(usize) -> bool] = &[
    &|__i| {
        let __elem = &__set_coll[__i];
        __report.silence();
        // <generated assertion for pattern 0 against __elem>
        let __matched = !__report.had_silenced_error();
        __report.unsilence();
        __matched
    },
    // ... one closure per pattern
];

// 3. Single runtime call
::assert_struct::__macro_support::set_match(
    __set_coll.len(),
    /*rest=*/ false,
    __preds,
    &mut __report,
    &__PATTERN_NODE_X,
);
```

`set_match` handles length checking, backtracking, and pushing to `__report` on failure. Nothing else lives in the macro.
