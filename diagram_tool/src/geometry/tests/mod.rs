use super::*;

#[allow(dead_code)]
pub const MIN_ZOOM: f64 = 0.1;
#[allow(dead_code)]
pub const MAX_ZOOM: f64 = 10.0;
#[allow(dead_code)]
pub fn clamp_zoom(zoom: f64) -> f64 {
    zoom.clamp(MIN_ZOOM, MAX_ZOOM)
}
#[allow(dead_code)]
pub const MIN_SIZE: f64 = 1.0;

pub mod bdd_tests_for_numeric_boundaries_bd_14y;
pub mod edg_031_edge_routing_stability_when_endpoints_swap;
pub mod geo_001_aabb_for_axis_aligned_rectangles;
pub mod geo_002_aabb_for_rotated_rectangles;
pub mod geo_003_stroke_width_inclusion_in_bounds;
pub mod geo_004_text_bounds_calculation;
pub mod geo_005_image_bounds_calculation;
pub mod geo_006_scale_around_anchor_point;
pub mod geo_007_rotate_around_center;
pub mod geo_008_resize_with_aspect_ratio_lock;
pub mod geo_009_combined_transform_chain;
pub mod geo_010_bounds_edge_cases;
pub mod geo_011_rotation_resize_composition;
pub mod geo_012_zoom_at_pointer;
pub mod geo_013_snap_lines_horizontal;
pub mod geo_014_snap_lines_vertical;
pub mod geo_015_grid_step;
pub mod geo_016_edge_routing_orthogonal;
pub mod geo_017_edge_routing_avoid_obstacle;
pub mod geo_018_fit_to_content;
pub mod geo_019_hit_test_with_margin;
pub mod geo_020_hit_test_rotated_shape;
pub mod geo_021_line_intersection;
pub mod geo_021_world_to_screen_round_trip;
pub mod geo_022_aabb_at_various_angles;
pub mod geo_023_rotation_then_resize_composition;
pub mod geo_024_resize_then_rotation_composition;
pub mod geo_025_repeated_tiny_transforms_rotation_drift;
pub mod geo_026_repeated_tiny_scales_scale_drift;
pub mod geo_027_camera_constraints_min_zoom;
pub mod geo_027_path_simplification_tests;
pub mod geo_028_camera_constraints_max_zoom;
pub mod geo_029_camera_pan_with_zoom;
pub mod geo_030_camera_world_to_screen_at_extremes;
pub mod geo_031_aabb_for_rotated_rectangle_at_cardinal_angles;
pub mod geo_032_aabb_includes_stroke_width_extended;
pub mod geo_033_line_bounds_include_arrowheads;
pub mod geo_034_cubic_bezier;
pub mod geo_034_quadratic_bezier;
pub mod geo_035_text_bounds_rtlemoji;
pub mod geo_trn_001_scale_around_anchor_point_nwnesesw;
pub mod geo_trn_002_rotate_around_selection_center;
pub mod geo_trn_003_rotate_around_custom_pivot;
pub mod geo_trn_004_minimum_size_clamp;
pub mod geo_trn_005_negative_scaling_flip_vs_clamp;
pub mod mul_001_rotate_around_center;
pub mod mul_002_mixed_rotation_combine;
pub mod mul_003_rotate_bound_edges_survive;
pub mod mul_004_rotate_360_no_drift;
pub mod mul_005_rotate_undoredo;
pub mod mul_016_rotate_asymmetric_selection;
pub mod path_tests;
pub mod property_based_tests;
pub mod property_based_tests_for_mul;
pub mod routing_tests;
pub mod transforms_tests;
