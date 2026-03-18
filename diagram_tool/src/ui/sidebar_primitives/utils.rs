pub fn merge_class(base: &str, class: Option<&str>) -> String {
    class.map_or_else(|| String::from(base), |extra| format!("{base} {extra}"))
}
