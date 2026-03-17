//! Editor state types for diagram documents.
//!
//! Contains `EditorState`, `EditorTheme`, and related types.

use super::types::OrderedFloat;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
#[serde(transparent)]
pub struct GridSize(pub f64);

impl Default for GridSize {
    fn default() -> Self {
        Self(20.0)
    }
}

impl Eq for GridSize {}

/// Editor theme options
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EditorTheme {
    Light,
    Dark,
    System,
}

const fn default_theme() -> EditorTheme {
    EditorTheme::System
}

const fn default_snap() -> bool {
    true
}

const fn default_show_grid() -> bool {
    true
}

/// Editor state for the diagram canvas
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub struct EditorState {
    pub camera_x: OrderedFloat,
    pub camera_y: OrderedFloat,
    pub zoom: OrderedFloat,
    #[serde(default)]
    pub grid_size: GridSize,
    #[serde(default = "default_snap")]
    pub snap_to_grid: bool,
    #[serde(default)]
    pub selected_items: im::HashSet<String>,
    #[serde(default)]
    pub edit_mode_target: Option<String>,
    #[serde(default)]
    pub editing_edge_id: Option<String>,
    #[serde(default = "default_theme")]
    pub theme: EditorTheme,
    #[serde(default = "default_show_grid")]
    pub show_grid: bool,
    #[serde(default)]
    pub minimap_visible: bool,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            camera_x: OrderedFloat::new_unchecked(0.0),
            camera_y: OrderedFloat::new_unchecked(0.0),
            zoom: OrderedFloat::new_unchecked(1.0),
            grid_size: GridSize::default(),
            snap_to_grid: true,
            selected_items: im::HashSet::new(),
            edit_mode_target: None,
            editing_edge_id: None,
            theme: default_theme(),
            show_grid: default_show_grid(),
            minimap_visible: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{EditorState, EditorTheme};

    #[test]
    fn default_editor_state_has_snap_and_grid_enabled() {
        let state = EditorState::default();
        assert!(state.snap_to_grid);
        assert!(state.show_grid);
    }

    #[test]
    fn editor_state_json_without_snap_flag_defaults_to_true() {
        let json = r#"{
            "camera_x": 0.0,
            "camera_y": 0.0,
            "zoom": 1.0,
            "grid_size": 20.0,
            "selected_items": [],
            "editing_edge_id": null,
            "theme": "system",
            "show_grid": true,
            "minimap_visible": false
        }"#;

        let state = serde_json::from_str::<EditorState>(json).ok();
        assert!(state.is_some_and(|parsed| parsed.snap_to_grid));
    }

    #[test]
    fn editor_state_serialization_roundtrip() {
        let state = EditorState::default();
        let json = serde_json::to_string(&state).unwrap();
        let parsed: EditorState = serde_json::from_str(&json).unwrap();
        assert_eq!(state, parsed);
    }

    #[test]
    fn editor_theme_all_variants_serialize() {
        for theme in [EditorTheme::Light, EditorTheme::Dark, EditorTheme::System] {
            let json = serde_json::to_string(&theme).unwrap();
            let parsed: EditorTheme = serde_json::from_str(&json).unwrap();
            assert_eq!(theme, parsed);
        }
    }
}
