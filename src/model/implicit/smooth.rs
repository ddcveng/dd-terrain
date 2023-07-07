use array_init::array_init;
use cgmath::Vector3;
use std::sync::Arc;

use crate::{
    config::WORLD_SIZE,
    infrastructure::texture::MaterialBlend,
    minecraft,
    model::{
        chunk::Chunk,
        polygonize::{polygonize, Mesh, Rectangle3D},
        rectangle::Rectangle,
        Coord, PlanarPosition, Position, Real, discrete::{WorldChunks, World},
    },
};

use super::sdf;
use super::normal;

pub fn get_density(world: &World, point: Position) -> Real {
    let chunks = world.get_chunks();
    evaluate_density_rigid(&chunks, point)
}

pub fn get_smooth_normal(world: &World, point: Position) -> Vector3<Real> {
    let chunks = world.get_chunks();
    let sdf = |p| evaluate_density_rigid(&chunks, p);

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

pub fn polygonize_chunk(chunks: &WorldChunks, chunk_index: usize) -> Mesh {
    let chunk = chunks[chunk_index].clone();
    let support_xz = chunk.position.get_global_position();

    let support_low_y = 40.0; // TODO: use MIN_Y
    let support_y_size = 40.0; // TODO: use full chunk height

    let support = Rectangle3D {
        position: Position::new(support_xz.x, support_low_y, support_xz.y),
        width: minecraft::BLOCKS_IN_CHUNK as Real,
        depth: minecraft::BLOCKS_IN_CHUNK as Real,
        height: support_y_size,
    };

    let density_func = |p| evaluate_density_rigid(&chunks, p);
    let material_func = |p| sample_materials(&chunks, p);

    polygonize(support, density_func, material_func)
}

const RIGID_BLOCK_SMOOTHNESS: Real = 1.0;
fn evaluate_density_rigid(model: &WorldChunks, point: Position) -> Real {
    let model_distance = -evaluate_density(model, point);
    let rigid_distance = distance_to_rigid_blocks(model, point);

    match rigid_distance {
        //Some(distance) => model_density.min(distance),
        Some(distance) => smooth_minimum(model_distance, distance, RIGID_BLOCK_SMOOTHNESS),
        None => model_distance,
    }
}

fn distance_to_rigid_blocks(chunks: &WorldChunks, point: Position) -> Option<Real> {
    let kernel = Kernel::new(point, 0.5);
    let kernel_box = kernel.get_bounding_rectangle();

    let intersected_chunks = chunks.iter().filter(|chunk| {
        chunk
            .get_bounding_rectangle()
            .intersect(kernel_box)
            .is_some()
    });

    let closest_rigid_block_per_chunk = intersected_chunks
        .map(|chunk| chunk.get_closest_rigid_block(point))
        .filter_map(|rigid_block_option| rigid_block_option);

    let Some(closest_rigid_block) =
        closest_rigid_block_per_chunk.fold(None, |min_dist, dist| match min_dist {
            None => Some(dist),
            Some(val) => {
                if dist.1 < val.1 {
                    Some(dist)
                } else {
                    min_dist
                }
            }
        }) 
    else {
        return None;
    };

    let block_position = closest_rigid_block.0.position;
    let block_local_point = point.zip(block_position, |k, b| k - b);

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

// Radius of the cube used as the convolution kernel used for density evaluation
// NOTE: if this is larger than 1.0, 1 block thick walls will disappear
const DENSITY_SIGMA: Coord = 0.9;
const KERNEL_VOLUME: Real = 8.0 * DENSITY_SIGMA * DENSITY_SIGMA * DENSITY_SIGMA;
const KERNEL_VOLUME_HALF: Real = KERNEL_VOLUME / 2.0;

// 2 * (material_volume / kernel_volume) - 1
// returns values in range [-1., 1.]
fn evaluate_density(model: &WorldChunks, point: Position) -> Real {
    let kernel = Kernel::new(point, DENSITY_SIGMA);
    return sample_volume(model, kernel) / KERNEL_VOLUME_HALF - 1.0;
}

fn sample_volume(chunks: &WorldChunks, kernel: Kernel) -> Real {
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
        let chunk_volume = chunk.get_chunk_intersection_volume(intersection_local, y_low, y_high);

        acc + chunk_volume
    })
}

// The smoothing process shrinks the world down a little
// so the material kernel shouldn't be much smaller than the density kernel.
// Otherwise artefacts may show up for places where the material kernel did not find
// any intersecting blocks
const MATERIAL_SIGMA: Coord = 0.6;

fn sample_materials(chunks: &WorldChunks, point: Position) -> MaterialBlend {
    let kernel = Kernel::new(point, MATERIAL_SIGMA);
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
            let chunk_volume = chunk.get_material_blend(intersection_local, y_low, y_high);

            blend.merge(chunk_volume);
            blend
        })
}
