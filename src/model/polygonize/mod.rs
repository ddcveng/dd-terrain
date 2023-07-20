use crate::imgui_wrapper::SmoothMeshOptions;
use crate::infrastructure::texture::MaterialBlend;

pub use self::marching_cubes::Mesh;
pub use self::marching_cubes::MeshVertex;
pub use self::marching_cubes::Rectangle3D;

use super::Coord;
use super::{Position, Real};

mod marching_cubes;

//pub enum PolygonizationMethod {
//    MarchingCubes,
//}

pub fn polygonize(
    support: Rectangle3D,
    density_func: impl Fn(Position) -> Real + Send + Sync,
    material_func: impl Fn(Position) -> MaterialBlend,
    options: PolygonizationOptions,
    //method: PolygonizationMethod,
) -> Mesh {
    self::marching_cubes::polygonize(support, density_func, material_func, options)
}

#[derive(Clone, Copy)]
pub struct PolygonizationOptions {
    // Radius of the cube used as the convolution kernel used for density evaluation
    // NOTE: if this is larger than 1.0, 1 block thick walls will disappear
    pub kernel_size: Coord,

    // The jump in quality between 1.0 and 0.9 is insane!
    //
    // This value should divide block size without remainder or weird artefacts occure when building
    pub marching_cubes_cell_size: Real,
    pub y_low_limit: Coord,
    pub y_size: Coord,
}

impl From<SmoothMeshOptions> for PolygonizationOptions {
    fn from(value: SmoothMeshOptions) -> Self {
        Self {
            kernel_size: kernel_size(value.smoothness_level),
            marching_cubes_cell_size: cell_size(value.mesh_resolution_level),
            y_low_limit: value.y_low_limit as Coord,
            y_size: value.y_size as Coord,
        }
    }
}

const SMOOTHNESS_STEP: Coord = 0.5;
fn kernel_size(smoothness: u8) -> Coord {
    match smoothness {
        0 | 1 => 0.5,
        2 => 0.9,
        n @ 3.. => 0.9 + ((n - 2) as Coord) * SMOOTHNESS_STEP,
    }
}

fn cell_size(mesh_resolution_level: u8) -> Real {
    let cells_per_vertex = match mesh_resolution_level {
        0 | 1 => 1,
        2 => 2,
        3 => 4,
        4.. => 8,
    };

    1.0 / (cells_per_vertex as Real)
}
