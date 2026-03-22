#[cfg(test)]
mod tests {
    use crate::history::History;
    use crate::ui::commands::alignment::*;
    use crate::ui::commands::tests_dsl::dsl::TestDsl;
    use diagram_models::document::NodeId;
    use dioxus::prelude::*;

    #[test]
    fn given_two_nodes_when_align_left_then_both_have_min_x() {
        let mut vdom = VirtualDom::new(|| {
            let doc = TestDsl::new()
                .with_node("n1", 10.0, 0.0, 100.0, 100.0, false)
                .with_node("n2", 50.0, 0.0, 100.0, 100.0, false)
                .with_selection(&["n1", "n2"])
                .build();
            let doc_signal = Signal::new(doc);
            let history_signal = Signal::new(History::new());

            let result = apply_align_selection(
                doc_signal,
                history_signal,
                AlignmentAxis::Horizontal,
                AlignmentMode::Start,
            );

            assert!(result);
            let n1 = doc_signal
                .read()
                .document
                .nodes
                .get(&NodeId::new("n1".to_string()))
                .unwrap()
                .clone();
            let n2 = doc_signal
                .read()
                .document
                .nodes
                .get(&NodeId::new("n2".to_string()))
                .unwrap()
                .clone();
            assert_eq!(n1.x.0, 10.0); // min x was 10.0
            assert_eq!(n2.x.0, 10.0);
            rsx! { div {} }
        });
        let _ = vdom.rebuild_in_place();
    }

    #[test]
    fn given_two_nodes_when_align_top_then_both_have_min_y() {
        let mut vdom = VirtualDom::new(|| {
            let doc = TestDsl::new()
                .with_node("n1", 0.0, 20.0, 100.0, 100.0, false)
                .with_node("n2", 0.0, 60.0, 100.0, 100.0, false)
                .with_selection(&["n1", "n2"])
                .build();
            let doc_signal = Signal::new(doc);
            let history_signal = Signal::new(History::new());

            let result = apply_align_selection(
                doc_signal,
                history_signal,
                AlignmentAxis::Vertical,
                AlignmentMode::Start,
            );

            assert!(result);
            let n1 = doc_signal
                .read()
                .document
                .nodes
                .get(&NodeId::new("n1".to_string()))
                .unwrap()
                .clone();
            let n2 = doc_signal
                .read()
                .document
                .nodes
                .get(&NodeId::new("n2".to_string()))
                .unwrap()
                .clone();
            assert_eq!(n1.y.0, 20.0); // min y was 20.0
            assert_eq!(n2.y.0, 20.0);
            rsx! { div {} }
        });
        let _ = vdom.rebuild_in_place();
    }

    #[test]
    fn given_locked_node_when_align_then_locked_node_not_moved() {
        let mut vdom = VirtualDom::new(|| {
            let doc = TestDsl::new()
                .with_node("n1", 10.0, 0.0, 100.0, 100.0, true) // locked
                .with_node("n2", 50.0, 0.0, 100.0, 100.0, false)
                .with_selection(&["n1", "n2"])
                .build();
            let doc_signal = Signal::new(doc);
            let history_signal = Signal::new(History::new());

            // Need at least 2 movable nodes to align, so this should return false and not do anything.
            let result = apply_align_selection(
                doc_signal,
                history_signal,
                AlignmentAxis::Horizontal,
                AlignmentMode::Start,
            );

            assert!(!result);
            rsx! { div {} }
        });
        let _ = vdom.rebuild_in_place();
    }

    #[test]
    fn given_two_nodes_when_align_right_then_both_have_max_right() {
        let mut vdom = VirtualDom::new(|| {
            let doc = TestDsl::new()
                .with_node("n1", 10.0, 0.0, 100.0, 100.0, false)
                .with_node("n2", 50.0, 0.0, 100.0, 100.0, false)
                .with_selection(&["n1", "n2"])
                .build();
            let doc_signal = Signal::new(doc);
            let history_signal = Signal::new(History::new());

            let result = apply_align_selection(
                doc_signal,
                history_signal,
                AlignmentAxis::Horizontal,
                AlignmentMode::End,
            );

            assert!(result);
            let n1 = doc_signal
                .read()
                .document
                .nodes
                .get(&NodeId::new("n1".to_string()))
                .unwrap()
                .clone();
            let n2 = doc_signal
                .read()
                .document
                .nodes
                .get(&NodeId::new("n2".to_string()))
                .unwrap()
                .clone();
            // max right is 50 + 100 = 150
            // n1 x = 150 - 100 = 50
            // n2 x = 150 - 100 = 50
            assert_eq!(n1.x.0, 50.0);
            assert_eq!(n2.x.0, 50.0);
            rsx! { div {} }
        });
        let _ = vdom.rebuild_in_place();
    }
}
