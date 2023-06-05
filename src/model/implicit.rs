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
}

pub fn sample_materials(model: &World, point: Position) -> MaterialBlend {
    let kernel = Kernel::new(point, MATERIAL_SIGMA);
    return model.sample_materials(kernel);
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

fn offset_position(pos: Position, dimension: Parameter) -> Position {
    let get_delta_for = |parameter: Parameter| {
        if parameter == dimension {
            EPSILON
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
    let f = |p| evaluate_density(model, p);

    gradient(f, point)
}

pub fn gradient(f: impl Fn(Position) -> Real, point: Position) -> Vector3<Real> {
    let dx = differentiate_dynamic(&f, point, Parameter::X);
    let dy = differentiate_dynamic(&f, point, Parameter::Y);
    let dz = differentiate_dynamic(&f, point, Parameter::Z);

    Vector3::new(dx, dy, dz)
}

// Only evaluates the function f 4 times instead of the regular 6
pub fn gradient_fast(f: impl Fn(Position) -> Real, point: Position) -> Vector3<Real> {
    let fx = f(point);
    let fnext_x = f(offset_position(point, Parameter::X));
    let fnext_y = f(offset_position(point, Parameter::Y));
    let fnext_z = f(offset_position(point, Parameter::Z));

    let differentiate_simple = |val| (val - fx) / EPSILON;
    let dx = differentiate_simple(fnext_x);
    let dy = differentiate_simple(fnext_y);
    let dz = differentiate_simple(fnext_z);

    Vector3::new(dx, dy, dz)
}
