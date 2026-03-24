use crate::test_utils::TestCategory;

#[cfg(kani)]
#[kani::proof]
fn test_category_expected_counts_are_correct() {
    assert_eq!(TestCategory::Sel.expected_count(), 25);
    assert_eq!(TestCategory::Clp.expected_count(), 10);
    assert_eq!(TestCategory::His.expected_count(), 13);
    assert_eq!(TestCategory::Mul.expected_count(), 37);
    assert_eq!(TestCategory::Sub.expected_count(), 34);
    assert_eq!(TestCategory::Edg.expected_count(), 35);
    assert_eq!(TestCategory::Cam.expected_count(), 12);
    assert_eq!(TestCategory::Geo.expected_count(), 30);
    assert_eq!(TestCategory::Snp.expected_count(), 10);
    assert_eq!(TestCategory::Io.expected_count(), 15);
    assert_eq!(TestCategory::Inp.expected_count(), 7);
}

#[cfg(kani)]
#[kani::proof]
fn test_total_expected_tests_is_228() {
    let total: usize = TestCategory::all().iter().map(|c| c.expected_count()).sum();
    assert_eq!(total, 228);
}

#[cfg(kani)]
#[kani::proof]
fn test_category_display_names() {
    assert_eq!(TestCategory::Sel.display_name(), "Selection");
    assert_eq!(TestCategory::Edg.display_name(), "Edge Binding");
    assert_eq!(TestCategory::Inp.display_name(), "Input (Touch/Stylus)");
}

#[cfg(kani)]
#[kani::proof]
fn test_category_all_returns_all_categories() {
    let all = TestCategory::all();
    assert_eq!(all.len(), 11);
}
