use dioxus::prelude::*;

#[test]
fn test_vdom() {
    let mut vdom = VirtualDom::new(|| rsx! { div { "hello" } });
    let mutations = vdom.rebuild_in_place();
    println!("{:?}", mutations);
}
