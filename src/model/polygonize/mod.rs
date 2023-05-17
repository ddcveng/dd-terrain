pub use self::marching_cubes::Mesh;
pub use self::marching_cubes::MeshVertex;
pub use self::marching_cubes::Rectangle3D;

use super::{Position, Real};

mod marching_cubes;

//pub enum PolygonizationMethod {
//    MarchingCubes,
//}

pub fn polygonize(
    density_func: impl Fn(Position) -> Real,
    support: Rectangle3D,
    //method: PolygonizationMethod,
) -> Mesh {
    self::marching_cubes::polygonize(support, density_func)
}
