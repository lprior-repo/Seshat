use base64::{engine::general_purpose, Engine as _};
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    if let Err(err) = run() {
        eprintln!("cargo:warning=icons index generation failed: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    println!("cargo:rerun-if-changed=resources/");

    let out_dir = env::var("OUT_DIR").map_err(|e| format!("OUT_DIR not set: {e}"))?;
    let resources_path = Path::new("resources");

    let mut icons: Vec<IconEntry> = Vec::new();

    if let Err(e) = scan_resources(resources_path, &mut icons) {
        eprintln!("Warning: Failed to scan resources: {e}");
    }

    icons.sort_by(|a, b| {
        a.provider
            .cmp(&b.provider)
            .then_with(|| a.category_path.cmp(&b.category_path))
            .then_with(|| a.icon_key.cmp(&b.icon_key))
    });

    let index = IconIndexJson {
        icons: icons.clone(),
        by_provider: build_by_provider(&icons),
    };

    let json =
        serde_json::to_string_pretty(&index).map_err(|e| format!("serialize index failed: {e}"))?;
    let json_path = Path::new(&out_dir).join("icons_index.json");
    fs::write(&json_path, &json)
        .map_err(|e| format!("write {} failed: {e}", json_path.display()))?;

    let rust_code = generate_rust_code();
    let rs_path = Path::new(&out_dir).join("icons_index.rs");
    fs::write(&rs_path, rust_code)
        .map_err(|e| format!("write {} failed: {e}", rs_path.display()))?;

    let icon_count = icons.len();
    let provider_count = index.by_provider.len();
    println!(
        "cargo:warning=Generated index for {icon_count} icons across {provider_count} providers"
    );

    Ok(())
}

#[derive(serde::Serialize, Clone)]
struct IconEntry {
    icon_key: String,
    provider: String,
    category_path: Vec<String>,
    file_relpath: String,
    display_name: String,
    search_terms: String,
    base64_data: String,
}

#[derive(serde::Serialize)]
struct IconIndexJson {
    icons: Vec<IconEntry>,
    by_provider: BTreeMap<String, Vec<String>>,
}

fn scan_resources(dir: &Path, icons: &mut Vec<IconEntry>) -> std::io::Result<()> {
    if !dir.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            scan_resources(&path, icons)?;
        } else if let Some(ext) = path.extension() {
            if ext == "png" || ext == "svg" {
                if let Some(icon) = parse_icon_path(&path) {
                    icons.push(icon);
                }
            }
        }
    }

    Ok(())
}

fn parse_icon_path(path: &Path) -> Option<IconEntry> {
    let relpath = path.strip_prefix("resources").ok()?;
    let relpath_str = relpath.to_str()?;

    let components: Vec<&str> = relpath
        .parent()?
        .components()
        .map(|c| c.as_os_str().to_str().unwrap_or(""))
        .collect();

    if components.is_empty() {
        return None;
    }

    let provider = components[0].to_string();
    let category_path: Vec<String> = components[1..]
        .iter()
        .map(std::string::ToString::to_string)
        .collect();

    let file_stem = path.file_stem()?.to_str()?;
    let icon_key = format!("{}/{file_stem}", relpath_str.rsplit_once('/')?.0);

    let display_name = title_case(file_stem);

    let search_terms = format!(
        "{} {} {} {}",
        icon_key,
        display_name,
        provider,
        category_path.join(" ")
    )
    .to_ascii_lowercase();

    let file_contents = fs::read(path).ok()?;
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("png");
    let mime = if ext.eq_ignore_ascii_case("svg") {
        "image/svg+xml"
    } else {
        "image/png"
    };

    let base64_data = format!(
        "data:{mime};base64,{}",
        general_purpose::STANDARD.encode(&file_contents)
    );

    Some(IconEntry {
        icon_key,
        provider,
        category_path,
        file_relpath: relpath_str.to_string(),
        display_name,
        search_terms,
        base64_data,
    })
}

fn title_case(s: &str) -> String {
    s.replace(['-', '_'], " ")
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            chars.next().map_or_else(String::new, |first| {
                first.to_uppercase().collect::<String>() + chars.as_str()
            })
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn build_by_provider(icons: &[IconEntry]) -> BTreeMap<String, Vec<String>> {
    let mut map: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for icon in icons {
        map.entry(icon.provider.clone())
            .or_default()
            .push(icon.icon_key.clone());
    }

    map
}

fn generate_rust_code() -> String {
    let code = r#"use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct IconMeta {
    pub icon_key: std::sync::Arc<str>,
    pub provider: std::sync::Arc<str>,
    pub category_path: Vec<std::sync::Arc<str>>,
    pub file_relpath: std::sync::Arc<str>,
    pub display_name: std::sync::Arc<str>,
    pub search_terms: std::sync::Arc<str>,
    pub base64_data: std::sync::Arc<str>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct IconIndex {
    pub all: Vec<IconMeta>,
    pub by_provider: BTreeMap<String, Vec<String>>,
    pub by_key: HashMap<String, IconMeta>,
}

impl IconIndex {
    /// Loads the icon index from the embedded JSON.
    ///
    /// # Panics
    ///
    /// Panics if the `icons_index.json` cannot be parsed.
    #[must_use]
    pub fn load() -> Self {
        let json: serde_json::Value = serde_json::from_str(include_str!("icons_index.json"))
            .expect("Failed to parse icons_index.json");
        let all: Vec<IconMeta> = serde_json::from_value(
            json.get("icons").cloned().unwrap_or_default()
        ).expect("Failed to parse icons array");
        let by_provider: BTreeMap<String, Vec<String>> = serde_json::from_value(
            json.get("by_provider").cloned().unwrap_or_default()
        ).expect("Failed to parse by_provider");
        let by_key: HashMap<String, IconMeta> = all
            .iter()
            .map(|icon| (icon.icon_key.to_string(), icon.clone()))
            .collect();
        Self { all, by_provider, by_key }
    }
    
    #[must_use]
    pub fn filter(&self, query: &str) -> Vec<&IconMeta> {
        if query.is_empty() {
            return self.all.iter().collect();
        }
        let query_lower = query.to_lowercase();
        self.all
            .iter()
            .filter(|icon| {
                icon.icon_key.to_lowercase().contains(&query_lower)
                    || icon.display_name.to_lowercase().contains(&query_lower)
            })
            .collect()
    }
    
    #[must_use]
    pub fn icons_by_provider(&self, provider: &str) -> Vec<&IconMeta> {
        self.by_provider
            .get(provider)
            .map(|keys| {
                keys.iter()
                    .filter_map(|key| self.by_key.get(key))
                    .collect()
            })
            .unwrap_or_default()
    }
}
"#;
    code.to_string()
}
