#[cfg(test)]
mod tests {
    use crate::history::History;
    use crate::ui::commands::distribution::*;
    use crate::ui::commands::tests_dsl::dsl::TestDsl;
    use diagram_models::document::NodeId;
    use dioxus::prelude::*;

    #[test]
    fn given_three_nodes_when_distribute_horizontal_then_evenly_spaced() {
        let mut vdom = VirtualDom::new(|| {
            let doc = TestDsl::new()
                .with_node("n1", 0.0, 0.0, 10.0, 10.0, false)
                .with_node("n2", 20.0, 0.0, 10.0, 10.0, false)
                .with_node("n3", 100.0, 0.0, 10.0, 10.0, false)
                .with_selection(&["n1", "n2", "n3"])
                .build();
            let doc_signal = Signal::new(doc);
            let history_signal = Signal::new(History::new());

            let result = apply_distribute_selection(
                doc_signal,
                history_signal,
                DistributionAxis::Horizontal,
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
            let n3 = doc_signal
                .read()
                .document
                .nodes
                .get(&NodeId::new("n3".to_string()))
                .unwrap()
                .clone();

            assert_eq!(n1.x.0, 0.0); // remains unchanged
            assert_eq!(n3.x.0, 100.0); // remains unchanged
                                       // total space = 110 - 0 = 110
                                       // sum of widths = 30
                                       // remaining space = 80
                                       // spacing = 80 / 2 = 40
                                       // n2 should be at 0 (n1 x) + 10 (n1 w) + 40 (spacing) = 50.0
            assert_eq!(n2.x.0, 50.0);
            rsx! { div {} }
        });
        let _ = vdom.rebuild_in_place();
    }

    #[test]
    fn given_fewer_than_three_nodes_when_distribute_then_returns_false() {
        let mut vdom = VirtualDom::new(|| {
            let doc = TestDsl::new()
                .with_node("n1", 0.0, 0.0, 10.0, 10.0, false)
                .with_node("n2", 20.0, 0.0, 10.0, 10.0, false)
                .with_selection(&["n1", "n2"])
                .build();
            let doc_signal = Signal::new(doc);
            let history_signal = Signal::new(History::new());

            let result = apply_distribute_selection(
                doc_signal,
                history_signal,
                DistributionAxis::Horizontal,
            );

            assert!(!result);
            rsx! { div {} }
        });
        let _ = vdom.rebuild_in_place();
    }
}
