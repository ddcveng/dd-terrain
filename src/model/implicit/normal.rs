use cgmath::{InnerSpace, Vector3};

use crate::model::{Position, Real};

const EPSILON: Real = 0.0001;

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

    let next_p = Position {
        x: pos.x + get_delta_for(Parameter::X),
        y: pos.y + get_delta_for(Parameter::Y),
        z: pos.z + get_delta_for(Parameter::Z),
    };

    next_p
}

// Only evaluates f 4 times instead of the regular 6
#[allow(unused)]
fn forward_gradient(f: impl Fn(Position) -> Real, point: Position) -> Vector3<Real> {
    let fx = f(point);
    let fnext_x = f(offset_position(point, Parameter::X, false));
    let fnext_y = f(offset_position(point, Parameter::Y, false));
    let fnext_z = f(offset_position(point, Parameter::Z, false));

    let differentiate_simple = |val| (val - fx) / EPSILON;
    let dx = differentiate_simple(fnext_x);
    let dy = differentiate_simple(fnext_y);
    let dz = differentiate_simple(fnext_z);

    Vector3::new(dx, dy, dz).normalize()
}

#[allow(unused)]
fn central_gradient(f: impl Fn(Position) -> Real, point: Position) -> Vector3<Real> {
    let fnext_x = f(offset_position(point, Parameter::X, false));
    let fprev_x = f(offset_position(point, Parameter::X, true));
    let dx = fnext_x - fprev_x;

    let fnext_y = f(offset_position(point, Parameter::Y, false));
    let fprev_y = f(offset_position(point, Parameter::Y, true));
    let dy = fnext_y - fprev_y;

    let fnext_z = f(offset_position(point, Parameter::Z, false));
    let fprev_z = f(offset_position(point, Parameter::Z, true));
    let dz = fnext_z - fprev_z;

    // Mathematically each term should be divided by 2*EPSILON but we are
    // normalizing the vector anyway, so it doesn't matter
    Vector3::new(dx, dy, dz).normalize()
}

pub fn gradient(f: impl Fn(Position) -> Real, point: Position) -> Vector3<Real> {
    central_gradient(f, point)
}
