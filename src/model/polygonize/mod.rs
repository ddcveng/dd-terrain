use crate::infrastructure::texture::MaterialBlend;

pub use self::marching_cubes::Mesh;
pub use self::marching_cubes::MeshVertex;
pub use self::marching_cubes::Rectangle3D;

use super::{Position, Real};

mod marching_cubes;

//pub enum PolygonizationMethod {
//    MarchingCubes,
//}

pub fn polygonize(
    support: Rectangle3D,
    density_func: impl Fn(Position) -> Real,
    material_func: impl Fn(Position) -> MaterialBlend,
    //method: PolygonizationMethod,
) -> Mesh {
    self::marching_cubes::polygonize(support, density_func, material_func)
}
