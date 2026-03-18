//! Operations for benchmarking.

use serde::{Deserialize, Serialize};

/// Diagram operations that can be benchmarked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Operation {
    /// Pan viewport
    Pan,
    /// Zoom in/out
    Zoom,
    /// Select node
    Select,
    /// Drag node
    Drag,
    /// Full frame render
    RenderFrame,
}

impl Operation {
    /// Returns all operations.
    #[must_use]
    pub const fn all() -> [Self; 5] {
        [
            Self::Pan,
            Self::Zoom,
            Self::Select,
            Self::Drag,
            Self::RenderFrame,
        ]
    }

    /// Returns the operation name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Pan => "pan",
            Self::Zoom => "zoom",
            Self::Select => "select",
            Self::Drag => "drag",
            Self::RenderFrame => "render_frame",
        }
    }

    /// Returns the expected complexity factor.
    #[must_use]
    pub const fn complexity_factor(self) -> f64 {
        match self {
            Self::Pan => 0.8,         // Relatively cheap
            Self::Zoom => 0.9,        // Slightly more expensive
            Self::Select => 0.7,      // Single node lookup
            Self::Drag => 1.0,        // Baseline
            Self::RenderFrame => 1.2, // Full render is most expensive
        }
    }
}

impl std::fmt::Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_operation_all() {
        let all = Operation::all();
        assert_eq!(all.len(), 5);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_operation_name() {
        assert_eq!(Operation::Pan.name(), "pan");
        assert_eq!(Operation::Zoom.name(), "zoom");
        assert_eq!(Operation::Select.name(), "select");
        assert_eq!(Operation::Drag.name(), "drag");
        assert_eq!(Operation::RenderFrame.name(), "render_frame");
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_operation_complexity() {
        // Select should be cheaper than render
        assert!(Operation::Select.complexity_factor() < Operation::RenderFrame.complexity_factor());
    }
}
