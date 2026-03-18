use crate::geometry::primitives::Point;
use crate::geometry::snap::mod_types::{NodeId, SnapNode, SnapResult, SnapType};

#[must_use]
pub fn snap_to_nodes(active: &SnapNode, targets: &[SnapNode], threshold: f64) -> SnapResult {
    if threshold < 0.0 || !threshold.is_finite() || targets.is_empty() {
        return SnapResult::inactive();
    }
    if !active.x.is_finite()
        || !active.y.is_finite()
        || !targets.iter().all(|t| t.x.is_finite() && t.y.is_finite())
    {
        return SnapResult::inactive();
    }

    let mut best: Option<(f64, f64, f64, NodeId, SnapType)> = None;

    let mut check_snap = |dist: f64, x: f64, y: f64, id: NodeId, snap_type: SnapType| {
        if dist <= threshold {
            let candidate = (x, y, dist, id, snap_type);
            let should_update = match &best {
                None => true,
                Some((_, _, best_dist, _, _)) => dist < *best_dist,
            };
            if should_update {
                best = Some(candidate);
            }
        }
    };

    for target in targets.iter().filter(|t| t.id != active.id) {
        let x_snaps = [
            (
                (active.center_x() - target.left()).abs(),
                target.left(),
                target.center_y(),
                SnapType::EdgeLeft,
            ),
            (
                (active.center_x() - target.center_x()).abs(),
                target.center_x(),
                target.center_y(),
                SnapType::CenterX,
            ),
            (
                (active.center_x() - target.right()).abs(),
                target.right(),
                target.center_y(),
                SnapType::EdgeRight,
            ),
        ];

        for (dist, x, y, snap_type) in x_snaps {
            check_snap(dist, x, y, target.id.clone(), snap_type);
        }

        let y_snaps = [
            (
                (active.center_y() - target.top()).abs(),
                target.center_x(),
                target.top(),
                SnapType::EdgeTop,
            ),
            (
                (active.center_y() - target.center_y()).abs(),
                target.center_x(),
                target.center_y(),
                SnapType::CenterY,
            ),
            (
                (active.center_y() - target.bottom()).abs(),
                target.center_x(),
                target.bottom(),
                SnapType::EdgeBottom,
            ),
        ];

        for (dist, x, y, snap_type) in y_snaps {
            check_snap(dist, x, y, target.id.clone(), snap_type);
        }
    }

    match best {
        Some((x, y, _, id, st)) => SnapResult::new(st, id, Point::new(x, y)),
        None => SnapResult::inactive(),
    }
}
