use crate::model::{Coord, Position, Real};
use cgmath::{Point3, Rad, Vector3};

pub const TITLE: &str = "dd-terrain";
pub const ASSETS_PATH: &str = r#"assets"#;
pub const DYNAMIC_WORLD: bool = true;

pub const FOVY: Rad<Real> = Rad(std::f64::consts::FRAC_PI_2);
pub const Z_NEAR: Real = 0.1;
pub const Z_FAR: Real = 100.;

pub const SPAWN_POINT: Position = Point3::new(314.09, 76.47, 288.93);
pub const SPAWN_DIR: Vector3<Coord> = Vector3::new(0.84, -0.41, 0.36);
//pub const SPAWN_POINT: Position = Point3::new(228.7, 66.77, -199.0);

pub const WORLD_SIZE: usize = 10;

pub const WORLD_FOLDER: &str = r#"assets/RavineDemo"#;

pub const CAMERA_MOVE_SPEED: Real = 5.0;
pub const SENSITIVITY: Real = 0.009;
pub const SPHERE_RADIUS: Real = 5.0; // TODO: is this needed?

pub const MULTIPASS: bool = true;
pub const LOCK_LEAVES: bool = true;
pub const FILTER_RIGID: bool = false;
