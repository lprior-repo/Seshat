use diagram_models::document::DiagramDocument;
fn main() {
    let doc = DiagramDocument::default();
    println!("{}", serde_json::to_string_pretty(&doc).unwrap());
}
