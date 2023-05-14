use cgmath::Point3;
use cgmath::Vector3;

use crate::discrete::World;

use super::rectangle::Rectangle;
use super::Coord;
use super::PlanarPosition;
use super::Position;
use super::Real;

// Radius of the cube used as the convolution kernel
const SIGMA: Coord = 1.0;
const KERNEL_VOLUME: Real = 8.0 * SIGMA * SIGMA * SIGMA;
const KERNEL_VOLUME_HALF: Real = KERNEL_VOLUME / 2.0;
const EPSILON: Real = 0.0001;

pub const KERNEL_SIZE: Coord = 2.0 * SIGMA;

#[derive(Copy, Clone)]
pub struct Kernel {
    pub position: Position,
}

impl Kernel {
    pub fn get_bounding_rectangle(&self) -> Rectangle {
        let origin = PlanarPosition {
            x: self.position.x - SIGMA,
            y: self.position.z - SIGMA,
        };

        Rectangle::square(origin, KERNEL_SIZE)
    }

    pub fn y_low(&self) -> Real {
        self.position.y - SIGMA
    }

    pub fn y_high(&self) -> Real {
        self.position.y + SIGMA
    }
}

pub fn evaluate_density(model: &World, point: Position) -> Real {
    let kernel = Kernel { position: point };
    return -evaluate_density_inner(model, kernel);
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

fn differentiate(f: impl Fn(Position) -> Real, x: Position, next_x: Position, h: Real) -> Real {
    let fx = f(x);
    let fnext_x = f(next_x);

    (fnext_x - fx) / h
}

fn differentiate_dynamic(f: impl Fn(Position) -> Real, p: Position, target: Parameter) -> Real {
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

    let dx = differentiate_dynamic(f, point, Parameter::X);
    let dy = differentiate_dynamic(f, point, Parameter::Y);
    let dz = differentiate_dynamic(f, point, Parameter::Z);

    Vector3::new(dx, dy, dz)
}
