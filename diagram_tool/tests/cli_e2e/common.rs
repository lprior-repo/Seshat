use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct E2eTest {
    pub dir: PathBuf,
    pub input: PathBuf,
}

impl E2eTest {
    pub fn setup(prefix: &str) -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0_u128, |d| d.as_nanos());
        let dir = std::env::temp_dir().join(format!("diagram-tool-{prefix}-{nanos}"));
        let _ = fs::create_dir_all(&dir);
        let input = dir.join("input.json");
        Self { dir, input }
    }

    pub fn write_sample(&self) {
        let content = r#"{"version":2,"revision":1,"document":{"nodes":{"n1":{"kind":"node","icon":"aws/compute/ec2","label":"API","x":10.0,"y":20.0,"width":80.0,"height":60.0,"locked":true,"parent":null,"dag_rank":null,"tags":[],"metadata":{},"z_index":0,"style":"box"},"n2":{"kind":"node","icon":"aws/database/rds","label":"DB","x":220.0,"y":40.0,"width":80.0,"height":60.0,"locked":true,"parent":null,"dag_rank":null,"tags":[],"metadata":{},"z_index":0,"style":"box"}},"edges":{"e1":{"source":"n1","target":"n2","label":"calls","style":"solid","arrowType":"default","label_offset_t":0.5,"color":null,"thickness":1.5,"directed":true,"bend_points":[],"tags":[],"metadata":{}}}},"editor_state":{"camera_x":0.0,"camera_y":0.0,"zoom":1.0,"grid_size":20.0,"snap_to_grid":true,"selected_items":[],"editing_edge_id":null,"theme":"system","show_grid":true,"minimap_visible":false}}"#;
        fs::write(&self.input, content).unwrap();
    }

    pub fn write_doc(&self, content: &str) {
        fs::write(&self.input, content).unwrap();
    }

    pub fn write_file(&self, name: &str, content: &str) -> PathBuf {
        let path = self.dir.join(name);
        fs::write(&path, content).unwrap();
        path
    }

    pub fn run_diagram_tool(&self, args: &[&str]) -> CommandResult {
        let output = Command::new(env!("CARGO_BIN_EXE_diagram_tool"))
            .args(args)
            .output()
            .unwrap();
        CommandResult { output }
    }

    pub fn validate(&self) -> CommandResult {
        self.run_diagram_tool(&["validate", "--input", &self.input.to_string_lossy()])
    }

    pub fn patch(&self, patch_file: &Path, output_file: &Path) -> CommandResult {
        self.run_diagram_tool(&[
            "patch",
            "--input",
            &self.input.to_string_lossy(),
            "--patch",
            &patch_file.to_string_lossy(),
            "--output",
            &output_file.to_string_lossy(),
        ])
    }
}

pub struct CommandResult {
    pub output: std::process::Output,
}

impl CommandResult {
    pub fn success(&self) -> bool {
        self.output.status.success()
    }

    pub fn jsonl_events(&self) -> Vec<Value> {
        let stdout_str = String::from_utf8(self.output.stdout.clone()).unwrap();
        stdout_str
            .lines()
            .filter_map(|line| serde_json::from_str::<Value>(line).ok())
            .collect()
    }

    pub fn has_event(&self, event_type: &str) -> bool {
        self.jsonl_events()
            .iter()
            .any(|v| v.get("event") == Some(&Value::String(event_type.to_string())))
    }

    pub fn has_error_event(&self, code: &str) -> bool {
        self.jsonl_events().iter().any(|v| {
            v.get("event") == Some(&Value::String("error".to_string()))
                && v.get("code") == Some(&Value::String(code.to_string()))
        })
    }

    pub fn has_finish_event_ok(&self, ok: bool) -> bool {
        self.jsonl_events().iter().any(|v| {
            v.get("event") == Some(&Value::String("finish".to_string()))
                && v.get("ok") == Some(&Value::Bool(ok))
        })
    }
}
