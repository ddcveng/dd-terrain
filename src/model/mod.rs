pub mod chunk;
pub mod common;
pub mod discrete;
pub mod implicit;
pub mod polygonize;
pub mod rectangle;

pub type Real = f64;
pub type Coord = f64;
pub type Position = cgmath::Point3<Coord>;
pub type PlanarPosition = cgmath::Point2<Coord>;
