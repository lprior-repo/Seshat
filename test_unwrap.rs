struct Foo;
fn main() {
    let x: Option<(i32, Foo)> = Some((1, Foo));
    let y = x.unwrap();
}
