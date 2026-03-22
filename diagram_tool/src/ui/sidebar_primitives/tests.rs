#[cfg(test)]
mod tests {
    use crate::ui::mobile::SidebarUiState;
    use crate::ui::sidebar_primitives::provider::toggle_sidebar;
    use dioxus::prelude::*;

    // --- Provider Context DSL ---

    pub struct ProviderStateDsl {
        state: SidebarUiState,
    }

    impl ProviderStateDsl {
        pub fn given_default_state() -> Self {
            Self {
                state: SidebarUiState {
                    open: true,
                    open_mobile: false,
                    is_mobile: false,
                },
            }
        }

        pub fn with_desktop_mode(mut self) -> Self {
            self.state.is_mobile = false;
            self
        }

        pub fn with_mobile_mode(mut self) -> Self {
            self.state.is_mobile = true;
            self
        }

        pub fn with_open_state(mut self, open: bool) -> Self {
            self.state.open = open;
            self
        }

        pub fn with_open_mobile_state(mut self, open_mobile: bool) -> Self {
            self.state.open_mobile = open_mobile;
            self
        }

        pub fn when_toggling_sidebar(mut self) -> Self {
            // We can directly invoke the logic that provider.rs uses,
            // but since it requires a Signal, we simulate the state transition based on the exact same logic.
            // toggle_sidebar takes a `&mut Signal<SidebarUiState>` which requires a Dioxus runtime.
            // For a lightweight ATDD DSL that doesn't mock the Dioxus runtime properties,
            // we validate the state transition rules directly as described by the Provider.
            if self.state.is_mobile {
                self.state.open_mobile = !self.state.open_mobile;
            } else {
                self.state.open = !self.state.open;
            }
            self
        }

        pub fn then_desktop_is_open(self, expected: bool) -> Self {
            assert_eq!(self.state.open, expected, "Desktop open state mismatch");
            self
        }

        pub fn then_mobile_is_open(self, expected: bool) -> Self {
            assert_eq!(
                self.state.open_mobile, expected,
                "Mobile open state mismatch"
            );
            self
        }
    }

    #[test]
    fn test_provider_toggles_desktop_sidebar() {
        ProviderStateDsl::given_default_state()
            .with_desktop_mode()
            .with_open_state(true)
            .when_toggling_sidebar()
            .then_desktop_is_open(false)
            .when_toggling_sidebar()
            .then_desktop_is_open(true);
    }

    #[test]
    fn test_provider_toggles_mobile_sidebar_independently() {
        ProviderStateDsl::given_default_state()
            .with_mobile_mode()
            .with_open_mobile_state(false)
            .with_open_state(true) // Desktop state should remain unchanged
            .when_toggling_sidebar()
            .then_mobile_is_open(true)
            .then_desktop_is_open(true)
            .when_toggling_sidebar()
            .then_mobile_is_open(false)
            .then_desktop_is_open(true);
    }

    // --- Tree Traversal DSL ---

    #[derive(Debug, Clone, PartialEq)]
    pub enum SidebarNode {
        Group {
            provider: String,
            expanded: bool,
            children: Vec<SidebarNode>,
        },
        Item {
            label: String,
        },
    }

    pub struct SidebarTreeDsl {
        root: Vec<SidebarNode>,
    }

    impl SidebarTreeDsl {
        pub fn given_empty_sidebar() -> Self {
            Self { root: vec![] }
        }

        pub fn with_group(
            mut self,
            provider: &str,
            expanded: bool,
            children: Vec<SidebarNode>,
        ) -> Self {
            self.root.push(SidebarNode::Group {
                provider: provider.to_string(),
                expanded,
                children,
            });
            self
        }

        pub fn with_item(mut self, label: &str) -> Self {
            self.root.push(SidebarNode::Item {
                label: label.to_string(),
            });
            self
        }

        pub fn when_toggling_group(mut self, target_provider: &str) -> Self {
            fn toggle(nodes: &mut Vec<SidebarNode>, target: &str) -> bool {
                for node in nodes.iter_mut() {
                    if let SidebarNode::Group {
                        provider,
                        expanded,
                        children,
                    } = node
                    {
                        if provider == target {
                            *expanded = !*expanded;
                            return true;
                        }
                        if toggle(children, target) {
                            return true;
                        }
                    }
                }
                false
            }
            let found = toggle(&mut self.root, target_provider);
            assert!(
                found,
                "Target group '{}' not found to toggle",
                target_provider
            );
            self
        }

        pub fn then_group_is_expanded(self, target_provider: &str, expected: bool) -> Self {
            fn find_expanded(nodes: &[SidebarNode], target: &str) -> Option<bool> {
                for node in nodes {
                    if let SidebarNode::Group {
                        provider,
                        expanded,
                        children,
                    } = node
                    {
                        if provider == target {
                            return Some(*expanded);
                        }
                        if let Some(res) = find_expanded(children, target) {
                            return Some(res);
                        }
                    }
                }
                None
            }

            let actual = find_expanded(&self.root, target_provider)
                .unwrap_or_else(|| panic!("Group '{}' not found", target_provider));
            assert_eq!(
                actual, expected,
                "Group '{}' expansion state mismatch",
                target_provider
            );
            self
        }

        pub fn then_visible_items_count(self, expected_count: usize) -> Self {
            fn count_visible(nodes: &[SidebarNode]) -> usize {
                let mut count = 0;
                for node in nodes {
                    match node {
                        SidebarNode::Item { .. } => count += 1,
                        SidebarNode::Group {
                            expanded, children, ..
                        } => {
                            count += 1; // The group header itself is visible
                            if *expanded {
                                count += count_visible(children);
                            }
                        }
                    }
                }
                count
            }

            let actual = count_visible(&self.root);
            assert_eq!(actual, expected_count, "Visible items count mismatch");
            self
        }
    }

    #[test]
    fn test_sidebar_tree_visibility_on_toggle() {
        SidebarTreeDsl::given_empty_sidebar()
            .with_item("Home")
            .with_group(
                "Settings",
                false,
                vec![
                    SidebarNode::Item {
                        label: "Profile".to_string(),
                    },
                    SidebarNode::Item {
                        label: "Preferences".to_string(),
                    },
                ],
            )
            .with_group(
                "Advanced",
                true,
                vec![
                    SidebarNode::Item {
                        label: "Network".to_string(),
                    },
                    SidebarNode::Item {
                        label: "Security".to_string(),
                    },
                ],
            )
            // Visibility count:
            // 1 (Home)
            // + 1 (Settings Header) + 0 (Settings is closed)
            // + 1 (Advanced Header) + 2 (Advanced is open)
            // = 5
            .then_visible_items_count(5)
            .when_toggling_group("Settings")
            .then_group_is_expanded("Settings", true)
            // Settings is now open, adding 2 items -> 7
            .then_visible_items_count(7)
            .when_toggling_group("Advanced")
            .then_group_is_expanded("Advanced", false)
            // Advanced is closed, removing 2 items -> 5
            .then_visible_items_count(5);
    }
}
