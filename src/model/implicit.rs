use cgmath::EuclideanSpace;
use cgmath::InnerSpace;
use cgmath::Point3;
use cgmath::Vector3;

use crate::discrete::World;
use crate::infrastructure::texture::MaterialBlend;

use super::rectangle::Rectangle;
use super::Coord;
use super::PlanarPosition;
use super::Position;
use super::Real;

// Radius of the cube used as the convolution kernel used for density evaluation
const DENSITY_SIGMA: Coord = 0.8;
const KERNEL_VOLUME: Real = 8.0 * DENSITY_SIGMA * DENSITY_SIGMA * DENSITY_SIGMA;
const KERNEL_VOLUME_HALF: Real = KERNEL_VOLUME / 2.0;
const EPSILON: Real = 0.0001;

// The smoothing process shrinks the world down a little
// so the material kernel shouldn't be much smaller than the density kernel.
// Otherwise artefacts may show up for places where the material kernel did not find
// any intersecting blocks
const MATERIAL_SIGMA: Coord = 0.6;

#[derive(Copy, Clone)]
pub struct Kernel {
    position: Position,
    radius: Coord,
}

impl Kernel {
    pub fn new(position: Position, radius: Coord) -> Self {
        Kernel { position, radius }
    }

    pub fn get_bounding_rectangle(&self) -> Rectangle {
        let origin = PlanarPosition {
            x: self.position.x - self.radius,
            y: self.position.z - self.radius,
        };

        Rectangle::square(origin, 2.0 * self.radius)
    }

    pub fn y_low(&self) -> Real {
        self.position.y - self.radius
    }

    pub fn y_high(&self) -> Real {
        self.position.y + self.radius
    }

    pub fn center(&self) -> Position {
        self.position
    }
}

pub fn sample_materials(model: &World, point: Position) -> MaterialBlend {
    let kernel = Kernel::new(point, MATERIAL_SIGMA);
    return model.sample_materials(kernel);
}

const RIGID_BLOCK_SMOOTHNESS: Real = 1.0;
pub fn evaluate_density_rigid(model: &World, point: Position) -> Real {
    let model_distance = -evaluate_density(model, point);
    let rigid_distance = model.distance_to_rigid_blocks(point);

    match rigid_distance {
        //Some(distance) => model_density.min(distance),
        Some(distance) => smooth_minimum(model_distance, distance, RIGID_BLOCK_SMOOTHNESS),
        None => model_distance,
    }
}

// Polynomial smooth min
// k controls the size of the region where the values are smoothed
//
// This version does not generalize to more than 2 dimensions
// and calling it multiple times with 2 arguments at a time is
// !NOT! order independent
fn smooth_minimum(a: Real, b: Real, k: Real) -> Real {
    let h = (k - (a - b).abs()).max(0.0) / k;
    a.min(b) - h * h * k * 0.25
}

pub fn evaluate_density(model: &World, point: Position) -> Real {
    let kernel = Kernel::new(point, DENSITY_SIGMA);
    return evaluate_density_inner(model, kernel);
}

// 2 * (material_volume / kernel_volume) - 1
// returns values in range [-1., 1.]
fn evaluate_density_inner(model: &World, kernel: Kernel) -> Real {
    return model.sample_volume(kernel) / KERNEL_VOLUME_HALF - 1.0;
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum Parameter {
    X = 0,
    Y = 1,
    Z = 2,
}

fn differentiate(f: &impl Fn(Position) -> Real, x: Position, next_x: Position, h: Real) -> Real {
    let fx = f(x);
    let fnext_x = f(next_x);

    (fnext_x - fx) / h
}

fn offset_position(pos: Position, dimension: Parameter, backwards: bool) -> Position {
    let sign = if backwards { -1.0 } else { 1.0 };
    let get_delta_for = |parameter: Parameter| {
        if parameter == dimension {
            sign * EPSILON
        } else {
            0.0
        }
    };

    let next_p = Point3 {
        x: pos.x + get_delta_for(Parameter::X),
        y: pos.y + get_delta_for(Parameter::Y),
        z: pos.z + get_delta_for(Parameter::Z),
    };

    next_p
}

fn differentiate_dynamic(f: &impl Fn(Position) -> Real, p: Position, target: Parameter) -> Real {
    let get_delta_for = |parameter: Parameter| {
        if parameter == target {
            EPSILON
        } else {
            0.0
        }
    };

    let next_p = Point3 {
        x: p.x + get_delta_for(Parameter::X),
        y: p.y + get_delta_for(Parameter::Y),
        z: p.z + get_delta_for(Parameter::Z),
    };

    differentiate(f, p, next_p, EPSILON)
}

pub fn get_gradient(model: &World, point: Position) -> Vector3<Real> {
    let f = |p| evaluate_density_rigid(model, p);

    gradient(f, point)
}

pub fn gradient(f: impl Fn(Position) -> Real, point: Position) -> Vector3<Real> {
    let dx = differentiate_dynamic(&f, point, Parameter::X);
    let dy = differentiate_dynamic(&f, point, Parameter::Y);
    let dz = differentiate_dynamic(&f, point, Parameter::Z);

    Vector3::new(dx, dy, dz)
}

// Only evaluates the function f 4 times instead of the regular 6
// Forward gradient
pub fn gradient_fast(f: impl Fn(Position) -> Real, point: Position) -> Vector3<Real> {
    let fx = f(point);
    let fnext_x = f(offset_position(point, Parameter::X, false));
    let fnext_y = f(offset_position(point, Parameter::Y, false));
    let fnext_z = f(offset_position(point, Parameter::Z, false));

    let differentiate_simple = |val| (val - fx) / EPSILON;
    let dx = differentiate_simple(fnext_x);
    let dy = differentiate_simple(fnext_y);
    let dz = differentiate_simple(fnext_z);

    Vector3::new(dx, dy, dz)
}

pub fn central_gradient(f: impl Fn(Position) -> Real, point: Position) -> Vector3<Real> {
    let fnext_x = f(offset_position(point, Parameter::X, false));
    let fprev_x = f(offset_position(point, Parameter::X, true));
    let dx = fnext_x - fprev_x;

    let fnext_y = f(offset_position(point, Parameter::Y, false));
    let fprev_y = f(offset_position(point, Parameter::Y, true));
    let dy = fnext_y - fprev_y;

    let fnext_z = f(offset_position(point, Parameter::Z, false));
    let fprev_z = f(offset_position(point, Parameter::Z, true));
    let dz = fnext_z - fprev_z;

    Vector3::new(dx, dy, dz).normalize()
}

const UNIT_CUBE_RADIUS: Real = 0.5;
// Expects position in the local space of the cube
pub fn sdf_unit_cube_exact(position: Position) -> Real {
    // Mirror the position to the positive octant and move the origin to the point (1, 1, 1) of
    // the unit cube
    let q = position.map(|p| p.abs() - UNIT_CUBE_RADIUS);

    let positive_q = q.map(|x| x.max(0.0));
    let outside_distance = positive_q.to_vec().magnitude();

    let max_q = q.x.max(q.y.max(q.z));
    let inside_distance = max_q.min(0.0);

    outside_distance + inside_distance
}
