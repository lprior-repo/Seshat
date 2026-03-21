use std::fmt::Write;

#[must_use]
pub fn xml_escape(s: &str) -> String {
    let mut escaped = String::with_capacity(s.len() * 2);
    for c in s.chars() {
        match c {
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '&' => escaped.push_str("&amp;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&apos;"),
            _ => escaped.push(c),
        }
    }
    escaped
}

pub fn render_text(svg: &mut String, x: f64, y: f64, label: &str) {
    let escaped_label = xml_escape(label);
    let _ = write!(
        svg,
        "<text x='{x}' y='{y}' text-anchor='middle' font-family='sans-serif' font-size='10'>{escaped_label}</text>"
    );
}
