use cgmath::Vector3;

use crate::{
    config,
    infrastructure::texture::MaterialBlend,
    minecraft,
    model::{
        common::{BlockType, MaterialSetup, RIGID_MATERIALS},
        discrete::{World, WorldChunks},
        polygonize::{polygonize, Mesh, PolygonizationOptions, Rectangle3D},
        rectangle::Rectangle,
        Coord, PlanarPosition, Position, Real,
    },
};

use super::normal;
use super::sdf;

pub fn get_density(world: &World, point: Position, kernel_size: Coord) -> Real {
    let chunks = world.get_chunks();
    evaluate_density_rigid(&chunks, point, kernel_size, &terrain_setup())
}

pub fn get_smooth_normal(world: &World, point: Position, kernel_size: Coord) -> Vector3<Real> {
    let chunks = world.get_chunks();
    let sdf = |p| evaluate_density_rigid(&chunks, p, kernel_size, &terrain_setup());

    normal::gradient(sdf, point)
}

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

    pub fn volume_half(&self) -> Real {
        4.0 * self.radius * self.radius * self.radius
    }

    pub fn y_low(&self) -> Real {
        self.position.y - self.radius
    }

    pub fn y_high(&self) -> Real {
        self.position.y + self.radius
    }

    pub fn center(&self) -> Position {
        self.position
    }
}

pub fn polygonize_chunk(
    chunks: &WorldChunks,
    chunk_index: usize,
    options: PolygonizationOptions,
) -> Mesh {
    let chunk = chunks[chunk_index].clone();
    let support_xz = chunk.position.get_global_position();

    let support_low_y = options.y_low_limit;
    let support_y_size = options.y_size;

    let support = Rectangle3D {
        position: Position::new(support_xz.x, support_low_y, support_xz.y),
        width: minecraft::BLOCKS_IN_CHUNK as Real,
        depth: minecraft::BLOCKS_IN_CHUNK as Real,
        height: support_y_size,
    };

    let terrain_mesh = {
        let terrain_setup = terrain_setup();

        let density_func =
            |p| evaluate_density_rigid(&chunks, p, options.kernel_size, &terrain_setup);
        let material_func = |p| {
            sample_materials(
                &chunks,
                p,
                material_sample_kernel_size(options.kernel_size),
                &terrain_setup,
            )
        };

        polygonize(support, density_func, material_func, options)
    };

    if config::MULTIPASS == false {
        return terrain_mesh;
    }

    let leaves_mesh = {
        let leaves_setup = MaterialSetup::include([BlockType::Leaves], []);

        let leaves_kernel_size = if config::LOCK_LEAVES {
            0.9
        } else {
            options.kernel_size
        };

        let density_func =
            |p| evaluate_density_rigid(&chunks, p, leaves_kernel_size, &leaves_setup);
        let material_func = |p| {
            sample_materials(
                &chunks,
                p,
                material_sample_kernel_size(leaves_kernel_size),
                &leaves_setup,
            )
        };

        polygonize(support, density_func, material_func, options)
    };

    Mesh::merge(&mut [terrain_mesh, leaves_mesh])
}

const RIGID_BLOCK_SMOOTHNESS: Real = 1.0;
fn evaluate_density_rigid(
    model: &WorldChunks,
    point: Position,
    kernel_size: Coord,
    material_setup: &MaterialSetup,
) -> Real {
    let model_distance = -evaluate_density(model, point, kernel_size, material_setup);
    let rigid_distance = distance_to_rigid_blocks(model, point, kernel_size, material_setup);

    match rigid_distance {
        //Some(distance) => model_distance.min(distance),
        Some(distance) => smooth_minimum(model_distance, distance, RIGID_BLOCK_SMOOTHNESS),
        None => model_distance,
    }
}

fn distance_to_rigid_blocks(
    chunks: &WorldChunks,
    point: Position,
    kernel_size: Coord,
    material_setup: &MaterialSetup,
) -> Option<Real> {
    if material_setup.no_rigid() {
        return None;
    }

    let kernel = Kernel::new(point, kernel_size);
    let kernel_box = kernel.get_bounding_rectangle();
    let y_low = kernel.y_low();
    let y_high = kernel.y_high();

    let closest_rigid_block = chunks
        .iter()
        .filter_map(|chunk| {
            chunk
                .get_bounding_rectangle()
                .intersect(kernel_box)
                .map(|intersection| (chunk, intersection))
        })
        .filter_map(|(chunk, intersection)| {
            chunk.get_closest_rigid_block(intersection, y_low, y_high, material_setup, point)
        })
        .min_by(|(_, _, dist1), (_, _, dist2)| dist1.total_cmp(dist2));

    let Some((rigid_block_position, _, _)) = closest_rigid_block else {
        return None;
    };

    let block_local_point = point.zip(rigid_block_position, |k, b| k - b);

    Some(sdf::unit_cube_exact(block_local_point))
}

// Polynomial smooth min
// k controls the size of the region where the values are smoothed
//
// This version does not generalize to more than 2 dimensions
// and calling it multiple times with 2 arguments at a time is
// !NOT! order independent
fn smooth_minimum(a: Real, b: Real, k: Real) -> Real {
    let h = (k - (a - b).abs()).max(0.0) / k;
    a.min(b) - h * h * k * 0.25
}

// 2 * (material_volume / kernel_volume) - 1
// returns values in range [-1., 1.]
fn evaluate_density(
    chunks: &WorldChunks,
    point: Position,
    kernel_size: Coord,
    material_setup: &MaterialSetup,
) -> Real {
    let kernel = Kernel::new(point, kernel_size);
    return sample_volume(chunks, kernel, material_setup) / kernel.volume_half() - 1.0;
}

fn sample_volume(chunks: &WorldChunks, kernel: Kernel, material_setup: &MaterialSetup) -> Real {
    let kernel_box = kernel.get_bounding_rectangle();
    let y_low = kernel.y_low();
    let y_high = kernel.y_high();

    chunks.iter().fold(0.0, |acc, chunk| {
        let chunk_box = chunk.get_bounding_rectangle();
        let Some(intersection) = chunk_box.intersect(kernel_box) else {
                return acc;
            };

        let offset = chunk.position.get_global_position().map(|coord| -coord);
        let intersection_local = intersection.offset_origin(offset);
        let chunk_volume =
            chunk.get_chunk_intersection_volume(intersection_local, y_low, y_high, material_setup);

        acc + chunk_volume
    })
}

fn sample_materials(
    chunks: &WorldChunks,
    point: Position,
    kernel_size: Coord,
    material_setup: &MaterialSetup,
) -> MaterialBlend {
    let kernel = Kernel::new(point, kernel_size);
    let kernel_box = kernel.get_bounding_rectangle();
    let y_low = kernel.y_low();
    let y_high = kernel.y_high();

    chunks
        .iter()
        .fold(MaterialBlend::new(), |mut blend, chunk| {
            let chunk_box = chunk.get_bounding_rectangle();
            let Some(intersection) = chunk_box.intersect(kernel_box) else {
                    return blend;
                };

            let offset = chunk.position.get_global_position().map(|coord| -coord);
            let intersection_local = intersection.offset_origin(offset);
            let chunk_volume =
                chunk.get_material_blend(intersection_local, y_low, y_high, material_setup);

            blend.merge(chunk_volume);
            blend
        })
}

fn terrain_setup() -> MaterialSetup {
    if config::MULTIPASS {
        MaterialSetup::exclude([BlockType::Leaves], RIGID_MATERIALS)
    } else {
        MaterialSetup::all_smooth(RIGID_MATERIALS)
    }
}

// The smoothing process shrinks the world down a little
// so the material kernel shouldn't be much smaller than the density kernel.
// Otherwise artefacts may show up for places where the material kernel did not find
// any intersecting blocks
fn material_sample_kernel_size(density_kernel_size: Coord) -> Coord {
    (density_kernel_size - 0.3).max(0.6)
}
