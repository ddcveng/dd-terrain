use array_init::array_init;
use itertools;
use std::cmp::max;
use std::cmp::min;

use crate::config;
use crate::config::WORLD_SIZE;
use crate::get_minecraft_chunk_position;
use crate::minecraft;

use super::chunk::{BlockData, Chunk, ChunkPosition};
use super::common::BlockType;
use super::implicit::Kernel;
use super::Position;
use super::Real;

// Represents a 2D grid of chunks
// Rows are parallel to the world x axis
// Columns are parallel to the world z axis
//
// Bigger indices correspond to bigger coordinates
//
// At any time only a part of the world is loaded, see config::WORLD_SIZE
// This strusts represents a "window" into the the world
// and can be used as a sliding window centered around the player
pub struct World {
    // Internal grid representation as a flat array
    chunks: [Chunk; WORLD_SIZE * WORLD_SIZE],

    // TODO: make private and get the world support to polygonize another way
    // Position of the center chunk in the world
    pub center: ChunkPosition,
}

fn get_difference_1d(region: i32, chunk: usize, new_region: i32, new_chunk: usize) -> i32 {
    if region == new_region {
        if chunk == new_chunk {
            return 0;
        }

        if chunk > new_chunk {
            return -1;
        }

        return 1;
    }

    if region > new_region {
        return -1;
    }

    return 1;
}

fn get_difference(original: &ChunkPosition, different: &ChunkPosition) -> (i32, i32) {
    let diff_x = get_difference_1d(
        original.region_x,
        original.chunk_x,
        different.region_x,
        different.chunk_x,
    );
    let diff_z = get_difference_1d(
        original.region_z,
        original.chunk_z,
        different.region_z,
        different.chunk_z,
    );

    (diff_x, diff_z)
}

fn clamp_chunk_index(i: i32) -> Option<usize> {
    if i >= 0 && (i as usize) < config::WORLD_SIZE {
        return Some(i as usize);
    }

    None
}

fn get_iterator(
    from: usize,
    to: usize,
    reverse: bool,
) -> itertools::Either<impl Iterator<Item = usize>, impl Iterator<Item = usize>> {
    if reverse {
        itertools::Either::Left((from..to).rev())
    } else {
        itertools::Either::Right(from..to)
    }
}

const OFFSET_FROM_CENTER: usize = config::WORLD_SIZE / 2;

impl World {
    pub fn new(position: Position) -> Self {
        let center_chunk_position = get_minecraft_chunk_position(position);

        // Get position of chunk that corresponds to 0,0 in the world grid
        let base_chunk_position = center_chunk_position
            .offset(-(OFFSET_FROM_CENTER as i32), -(OFFSET_FROM_CENTER as i32));

        World {
            chunks: array_init(|index| {
                let x = index % config::WORLD_SIZE;
                let z = index / config::WORLD_SIZE;
                let chunk_position = base_chunk_position.offset(x as i32, z as i32);

                minecraft::get_chunk(chunk_position)
            }),
            center: center_chunk_position,
        }
    }

    pub fn get_block_data(&self) -> Vec<BlockData> {
        let mut blocks = Vec::<BlockData>::new();

        for chunk in &self.chunks {
            let mut chunk_blocks = chunk.get_block_data();
            blocks.append(&mut chunk_blocks);
        }

        blocks
    }

    // Returns true if a new part of the world was loaded
    pub fn update(&mut self, new_position: Position) -> bool {
        let new_center_chunk = get_minecraft_chunk_position(new_position);
        if self.center == new_center_chunk {
            return false;
        }

        // Get the direction of change
        let (direction_x, direction_z) = get_difference(&self.center, &new_center_chunk);
        self.center = new_center_chunk;

        // Update the world matrix by either shifting chunks based on the direction, or loading
        // needed chunks
        let reverse_x = direction_x < 0;
        let reverse_z = direction_z < 0;

        for z in get_iterator(0, WORLD_SIZE, reverse_z) {
            for x in get_iterator(0, WORLD_SIZE, reverse_x) {
                let next_x = x as i32 + direction_x;
                let next_z = z as i32 + direction_z;
                if let Some(next_x) = clamp_chunk_index(next_x) {
                    if let Some(next_z) = clamp_chunk_index(next_z) {
                        let current_index = z * config::WORLD_SIZE + x;
                        let next_index = next_z * config::WORLD_SIZE + next_x;

                        self.chunks.swap(current_index, next_index);
                        continue;
                    }
                }

                let original_x =
                    min(max(x as i32 - direction_x, 0), WORLD_SIZE as i32 - 1) as usize;
                let original_z =
                    min(max(z as i32 - direction_z, 0), WORLD_SIZE as i32 - 1) as usize;
                let current_position =
                    &self.chunks[original_z * config::WORLD_SIZE + original_x].position;
                let position_to_load = current_position.offset(direction_x, direction_z);

                let new_chunk = minecraft::get_chunk(position_to_load);
                self.chunks[z * config::WORLD_SIZE + x] = new_chunk;
            }
        }

        return true;
    }

    pub fn get_block(&self, position: Position) -> Option<BlockType> {
        let chunk_position = get_minecraft_chunk_position(position);
        let chunk = self
            .chunks
            .iter()
            .find(|chunk| chunk.position == chunk_position);

        let Some(chunk) = chunk else {
            return None
        };

        let (block_x, block_z) = Chunk::get_block_coords(position.x, position.z);
        Some(chunk.get_block(block_x, position.y as isize, block_z))
    }

    pub fn sample_volume(&self, kernel: Kernel) -> Real {
        let kernel_box = kernel.get_bounding_rectangle();
        let y_low = kernel.y_low();
        let y_high = kernel.y_high();

        self.chunks.iter().fold(0.0, |acc, chunk| {
            let chunk_box = chunk.get_bounding_rectangle();
            let Some(intersection) = chunk_box.intersect(kernel_box) else {
                return acc;
            };

            let offset = chunk.position.get_global_position().map(|coord| -coord);
            let intersection_local = intersection.offset_origin(offset);
            let chunk_volume =
                chunk.get_chunk_intersection_volume(intersection_local, y_low, y_high);

            acc + chunk_volume
        })
    }
}
