use crate::model::{Position, Real};
use cgmath::{Point3, Rad};

pub const TITLE: &str = "dd-terrain";
pub const ASSETS_PATH: &str = r#"/home/ddcveng/projects/dd-terrain/assets"#;
pub const DYNAMIC_WORLD: bool = false;

pub const FOVY: Rad<Real> = Rad(std::f64::consts::FRAC_PI_2);
pub const Z_NEAR: Real = 0.1;
pub const Z_FAR: Real = 50.;

pub const SPAWN_POINT: Position = Point3::new(201.7, 64.77, 172.0);

pub const WORLD_SIZE: usize = 5;

pub const WORLD_FOLDER: &str = r#"/home/ddcveng/.minecraft/saves/jahoda"#;

pub const CAMERA_MOVE_SPEED: Real = 5.0;
pub const SENSITIVITY: Real = 0.009;
pub const SPHERE_RADIUS: Real = 5.0; // TODO: is this needed?
