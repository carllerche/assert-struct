use assert_struct::assert_struct;

struct Foo {
    bar: ((usize, usize), usize),
}

fn main() {
    let actual = Foo { bar: ((0, 0), 0) };
    assert_struct!(actual, _ { bar.0.0: 0, .. });
}
