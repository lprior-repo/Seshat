use im::HashMap;

fn main() {
    let mut m = HashMap::new();
    m.insert(1, 2);
    let m2 = m.update(1, 3);
    println!("{:?}", m2);
}
