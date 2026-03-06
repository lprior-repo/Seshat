use dioxus::prelude::*;
fn test(evt: Event<MouseData>) {
    let coords = evt.data.coordinates().element();
    println!("{:?}", coords);
}
fn main() {}
