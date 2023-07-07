use cgmath::{EuclideanSpace, InnerSpace};

use crate::model::{Position, Real};

const UNIT_CUBE_RADIUS: Real = 0.5;
// Expects position in the local space of the cube
pub fn unit_cube_exact(position: Position) -> Real {
    // Mirror the position to the positive octant and move the origin to the point (1, 1, 1) of
    // the unit cube
    let q = position.map(|p| p.abs() - UNIT_CUBE_RADIUS);

    let positive_q = q.map(|x| x.max(0.0));
    let outside_distance = positive_q.to_vec().magnitude();

    let max_q = q.x.max(q.y.max(q.z));
    let inside_distance = max_q.min(0.0);

    outside_distance + inside_distance
}
