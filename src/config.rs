use cgmath::{Point3, Rad};

pub const TITLE: &str = "dd-terrain";

pub const FOVY: Rad<f32> = Rad(std::f32::consts::FRAC_PI_2);
pub const Z_NEAR: f32 = 0.1;
pub const Z_FAR: f32 = 50.;

// -290 67 139
pub const SPAWN_POINT: Point3<f32> = Point3::new(-360., 100.0, 42.0);

pub const WORLD_SIZE: usize = 5;

// banan
pub const WORLD_FOLDER: &str = r#"/home/dd/.minecraft/saves/banan"#;

pub const CAMERA_MOVE_SPEED: f32 = 5.0;
pub const SENSITIVITY: f32 = 0.005;
pub const SPHERE_RADIUS: f32 = 5.0; // TODO: is this needed?
