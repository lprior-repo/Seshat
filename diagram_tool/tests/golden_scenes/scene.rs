#![allow(dead_code)]
#![allow(
    clippy::unwrap_used,
    clippy::panic,
    clippy::module_inception,
    clippy::let_unit_value,
    clippy::redundant_pattern_matching,
    unused_variables,
    unused_imports
)]
use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

/// A strongly typed wrapper for a loaded scene document.
/// Enforces valid document structure at the boundary (DDD: Parse, don't validate).
pub struct Scene {
    pub doc: Value,
}

impl Scene {
    pub fn load(name: &str) -> Self {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name);
        let content = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("Failed to read fixture '{}': {}", name, e));
        let doc: Value = serde_json::from_str(&content)
            .unwrap_or_else(|e| panic!("Failed to parse fixture '{}': {}", name, e));

        // Boundary validation: ensure core structures exist
        assert!(doc.get("document").is_some(), "Missing document field");
        assert!(
            doc["document"].get("nodes").is_some(),
            "Missing nodes field"
        );
        assert!(
            doc["document"].get("edges").is_some(),
            "Missing edges field"
        );

        Self { doc }
    }

    pub fn version(&self) -> u64 {
        self.doc["version"].as_u64().unwrap_or(0)
    }

    pub fn revision(&self) -> u64 {
        self.doc["revision"].as_u64().unwrap_or(0)
    }

    pub fn nodes(&self) -> &serde_json::Map<String, Value> {
        self.doc["document"]["nodes"].as_object().unwrap()
    }

    pub fn edges(&self) -> &serde_json::Map<String, Value> {
        self.doc["document"]["edges"].as_object().unwrap()
    }

    pub fn node(&self, id: &str) -> &Value {
        self.nodes()
            .get(id)
            .unwrap_or_else(|| panic!("Missing node {}", id))
    }

    pub fn edge(&self, id: &str) -> &Value {
        self.edges()
            .get(id)
            .unwrap_or_else(|| panic!("Missing edge {}", id))
    }

    pub fn assert_unique_ids(&self) {
        let n = self.nodes();
        let e = self.edges();
        assert_eq!(
            n.len(),
            n.keys().collect::<HashSet<_>>().len(),
            "Duplicate node IDs"
        );
        assert_eq!(
            e.len(),
            e.keys().collect::<HashSet<_>>().len(),
            "Duplicate edge IDs"
        );
    }
}
