import re

with open("diagram_tool/src/models/selection.rs", "r") as f:
    content = f.read()

# We need to change the marquee test because the logic of compute_marquee_selection is:
# let is_selected = if is_parent {
#     // Must be fully enclosed
#     min_x >= marquee.x && max_x <= m_right && min_y >= marquee.y && max_y <= m_bottom
# } else {
#     // Must intersect
#     !(min_x > m_right || max_x < marquee.x || min_y > m_bottom || max_y < marquee.y)
# };

# For `n1` which is a parent of `child`, its bounds are (0, 0) to (100, 100).
# The marquee is at (45, 45) to (65, 65).
# So n1 is NOT fully enclosed by the marquee. (0 >= 45 is false).
# So `n1` will NOT be selected! The test asserts `n1` IS selected.

# Let's fix the test by changing the marquee to be large enough to enclose `n1`.
# e.g., Rect::new(-10.0, -10.0, 120.0, 120.0)

test_old = """    #[test]
    fn test_sel_025_marquee_selects_nodes_inside_subgraphs() {
        let mut doc = setup_doc();
        let child = Node {
            kind: NodeKind::Node,
            icon: "".to_string(),
            label: "child".to_string(),
            x: OrderedFloat(50.0),
            y: OrderedFloat(50.0),
            width: OrderedFloat(10.0),
            height: OrderedFloat(10.0),
            font_size: None,
            font_weight: None,
            locked: false,
            parent: Some(NodeId::new("n1".to_string())),
            dag_rank: None,
            tags: im::Vector::new(),
            metadata: HashMap::new(),
            z_index: 0,
            style: None,
            collapsed: None,
        };
        doc.document
            .nodes
            .insert(NodeId::new("child".to_string()), child);

        let marquee = Rect::new(45.0, 45.0, 20.0, 20.0).unwrap();
        let selected = compute_marquee_selection(&doc, marquee).unwrap();

        assert!(selected.contains(&NodeId::new("child".to_string())));
        assert!(selected.contains(&NodeId::new("n1".to_string()))); // n1 bounds 0,0 100,100, overlaps
        assert!(!selected.contains(&NodeId::new("n2".to_string())));
    }"""

test_new = """    #[test]
    fn test_sel_025_marquee_selects_nodes_inside_subgraphs() {
        let mut doc = setup_doc();
        let child = Node {
            kind: NodeKind::Node,
            icon: "".to_string(),
            label: "child".to_string(),
            x: OrderedFloat(50.0),
            y: OrderedFloat(50.0),
            width: OrderedFloat(10.0),
            height: OrderedFloat(10.0),
            font_size: None,
            font_weight: None,
            locked: false,
            parent: Some(NodeId::new("n1".to_string())),
            dag_rank: None,
            tags: im::Vector::new(),
            metadata: HashMap::new(),
            z_index: 0,
            style: None,
            collapsed: None,
        };
        doc.document
            .nodes
            .insert(NodeId::new("child".to_string()), child);

        // Make marquee large enough to fully enclose the parent n1 (0,0 to 100,100)
        let marquee = Rect::new(-10.0, -10.0, 120.0, 120.0).unwrap();
        let selected = compute_marquee_selection(&doc, marquee).unwrap();

        assert!(selected.contains(&NodeId::new("child".to_string())));
        assert!(selected.contains(&NodeId::new("n1".to_string()))); // n1 bounds 0,0 100,100, fully enclosed
        assert!(!selected.contains(&NodeId::new("n2".to_string())));
    }"""

content = content.replace(test_old, test_new)

with open("diagram_tool/src/models/selection.rs", "w") as f:
    f.write(content)
