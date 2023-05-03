use std::cmp::max;
use std::cmp::min;

use crate::config;
use crate::config::WORLD_SIZE;
use crate::get_minecraft_chunk_position;
use crate::minecraft;
use array_init::array_init;
use cgmath::Point2;
use cgmath::Point3;
use glium::implement_vertex;
use itertools;

use super::implicit;
use super::implicit::Kernel;
use super::rectangle::Rectangle;

// TODO: is 1 byte for block type enough?
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum BlockType {
    Air = 0,
    Dirt = 1,
    Grass = 2,
    Stone = 3,
    Wood = 4,
    Sand = 5,
    Unknown = 255,
}

#[derive(Clone, Copy)]
pub struct BlockData {
    offset: [f32; 3],
    instance_color: [f32; 3],
    height: u32,
    //block_type: u8,
}
implement_vertex!(BlockData, offset, instance_color, height);

#[derive(Clone, Copy)]
struct MaterialLayer {
    material: BlockType,
    height: u32,
    base_height: isize,
}

// TODO: limit tower height to whatever minecraft allows
// Stores continuous layers of blocks,
// there can be gaps of air between layers, these are not stored.
//
// Block layers are ordered from lowest to highest y coordinate
struct MaterialTower {
    pub data: Vec<MaterialLayer>,
}

impl MaterialTower {
    pub fn new() -> Self {
        MaterialTower { data: Vec::new() }
    }

    pub fn get_block_at_y(&self, y: isize) -> BlockType {
        let layer = self.data.iter().find(|layer| {
            y >= layer.base_height && ((y - layer.base_height) as u32) < layer.height
        });

        if let Some(layer) = layer {
            return layer.material;
        }

        // If there is no block recorded at this height, assume its air
        return BlockType::Air;
    }

    pub fn get_layers_in_range(&self, y_low: f32, y_high: f32) -> Vec<(BlockType, f32)> {
        let layers_in_range = self.data.iter().filter_map(|layer| {
            let layer_low = (layer.base_height as f32).max(y_low);
            let layer_high = (layer.base_height as f32 + layer.height as f32).min(y_high);
            let layer_in_range = layer_high > layer_low;
            if !layer_in_range {
                return None;
            }

            let new_height = layer_high - layer_low;
            Some((layer.material, new_height))
        });

        return layers_in_range.collect();
    }

    pub fn push(&mut self, block: BlockType, base_height: isize) {
        // We do not want to store Air blocks for now
        debug_assert!(block != BlockType::Air);

        let extend_top_layer = match self.data.last() {
            Some(layer) => {
                // Extend the layer if materials match and there is no air gap between the layers
                layer.material == block
                    && (layer.base_height + layer.height as isize) == base_height
            }
            None => false,
        };

        if extend_top_layer {
            // Should always be Some(..) if the check above passed
            if let Some(top_layer) = self.data.last_mut() {
                top_layer.height += 1;
            } else {
                println!("Something weird is going on..");
            }
            return;
        }

        let segment = MaterialLayer {
            material: block,
            height: 1,
            base_height,
        };

        self.data.push(segment);
    }
}

fn get_block_color(block_type: BlockType) -> [f32; 3] {
    match block_type {
        BlockType::Grass => [0.09, 0.4, 0.05],
        BlockType::Dirt => [0.36, 0.09, 0.05],
        BlockType::Stone => [0.6, 0.6, 0.6],
        BlockType::Sand => [0.76, 0.69, 0.5],
        _ => [1.0, 0.0, 0.0],
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ChunkPosition {
    pub region_x: i32,
    pub region_z: i32,

    // These can only have values from 0 to minecraft::CHUNKS_IN_REGION - 1
    pub chunk_x: usize,
    pub chunk_z: usize,
}

impl ChunkPosition {
    pub fn get_global_position_in_chunks(&self) -> (i32, i32) {
        let global_x = self.region_x * (minecraft::CHUNKS_IN_REGION as i32) + (self.chunk_x as i32);
        let global_z = self.region_z * (minecraft::CHUNKS_IN_REGION as i32) + (self.chunk_z as i32);

        (global_x, global_z)
    }

    pub fn get_global_position(&self) -> Point2<f32> {
        let (chunk_x, chunk_z) = self.get_global_position_in_chunks();
        Point2::new(
            (chunk_x * minecraft::BLOCKS_IN_CHUNK as i32) as f32,
            (chunk_z * minecraft::BLOCKS_IN_CHUNK as i32) as f32,
        )
    }

    pub fn offset(&self, offset_x: i32, offset_z: i32) -> Self {
        let mut chunk_x = offset_x + self.chunk_x as i32;
        let mut chunk_z = offset_z + self.chunk_z as i32;

        // TODO: move into a function
        let region_x = if chunk_x < 0 {
            let region_offset = chunk_x.abs() / 32;

            chunk_x += 32;
            self.region_x - region_offset - 1
        } else if chunk_x > 31 {
            let region_offset = chunk_x.abs() / 32;

            chunk_x %= 32;
            self.region_x + region_offset
        } else {
            self.region_x
        };

        let region_z = if chunk_z < 0 {
            let region_offset = chunk_z.abs() / 32;

            chunk_z += 32;
            self.region_z - region_offset - 1
        } else if chunk_z > 31 {
            let region_offset = chunk_z.abs() / 32;

            chunk_z %= 32;
            self.region_z + region_offset
        } else {
            self.region_z
        };

        ChunkPosition {
            region_x,
            region_z,
            chunk_x: chunk_x as usize,
            chunk_z: chunk_z as usize,
        }
    }
}

const EPSILON: f32 = 0.0001;

fn get_block_coord(world_coord: f32) -> usize {
    let negative = world_coord < 0.0;

    let coord_positive = if negative { -world_coord } else { world_coord };

    // coord x.yy is still within the block at x -> floor coord_positive
    let block_coord = (coord_positive + EPSILON) as usize % minecraft::BLOCKS_IN_CHUNK;

    if negative {
        minecraft::BLOCKS_IN_CHUNK - block_coord - 1
    } else {
        block_coord
    }
}

fn get_block_portion_in_range(block_start: usize, range_start: f32, range_end: f32) -> f32 {
    let block_end = (block_start + 1) as f32; // everything is calculated in blocks
    let block_start: f32 = block_start as f32;

    let no_overlap = block_end < range_start || block_start > range_end;
    if no_overlap {
        return 0.0;
    }

    let mut portion: f32 = 1.0; // whole block is in range

    // cut portion from start of block
    if block_start < range_start {
        portion -= range_start - block_start;
    }

    if block_end > range_end {
        portion -= block_end - range_end;
    }

    portion
}

const CHUNK_SIZE: f32 = minecraft::BLOCKS_IN_CHUNK as f32;

// A chunks is a 16*y*16 region of blocks
pub struct Chunk {
    // A column major grid of towers
    data: [[MaterialTower; minecraft::BLOCKS_IN_CHUNK]; minecraft::BLOCKS_IN_CHUNK],

    // This is the position of the bottom left corner of the chunk from a top down view
    position: ChunkPosition,
}

impl Chunk {
    pub fn new(chunk_position: ChunkPosition) -> Self {
        Chunk {
            data: array_init(|_x| array_init(|_y| MaterialTower::new())),
            position: chunk_position,
        }
    }

    // Push block on top of the material tower at x, z
    pub fn push_block(&mut self, x: usize, z: usize, base_height: isize, block: BlockType) {
        let tower = &mut self.data[x][z];
        tower.push(block, base_height);
    }

    pub fn get_block_data(&self) -> Vec<BlockData> {
        let mut blocks = Vec::<BlockData>::new();
        let (chunk_global_x, chunk_global_z) = self.position.get_global_position_in_chunks();
        let global_offset_blocks_x = chunk_global_x * (minecraft::BLOCKS_IN_CHUNK as i32);
        let global_offset_blocks_z = chunk_global_z * (minecraft::BLOCKS_IN_CHUNK as i32);

        let mut x = 0;
        for col in &self.data {
            let mut z = 0;
            for tower in col {
                for segment in &tower.data {
                    let x_offset_blocks = global_offset_blocks_x + x;
                    let z_offset_blocks = global_offset_blocks_z + z;
                    let block_data = BlockData {
                        offset: [
                            x_offset_blocks as f32,
                            segment.base_height as f32,
                            z_offset_blocks as f32,
                        ],
                        instance_color: get_block_color(segment.material),
                        height: segment.height,
                    };
                    blocks.push(block_data);
                }
                z += 1;
            }
            x += 1;
        }

        blocks
    }

    pub fn get_block_coords(x: f32, z: f32) -> (usize, usize) {
        let block_x = get_block_coord(x);
        let block_z = get_block_coord(z);

        (block_x, block_z)
    }

    pub fn get_block(&self, x: usize, y: isize, z: usize) -> BlockType {
        let tower = &self.data[x][z];

        tower.get_block_at_y(y)
    }

    // Intersection is a rectangle local to the chunk - its origin is in chunk local coordinates
    // and the whole rectangle fits inside the chunk
    pub fn get_chunk_intersection_volume(
        &self,
        intersection_xz: Rectangle,
        y_low: f32,
        y_high: f32,
    ) -> f32 {
        let intersection_start_index_x = get_block_coord(intersection_xz.left());
        let intersection_start_index_z = get_block_coord(intersection_xz.bottom());

        let intersection_end_index_x = min(
            minecraft::BLOCKS_IN_CHUNK,
            (intersection_xz.right() - EPSILON).ceil() as usize,
        );
        let intersection_end_index_z = min(
            minecraft::BLOCKS_IN_CHUNK,
            (intersection_xz.top() - EPSILON).ceil() as usize,
        );

        let mut volume: f32 = 0.0;
        // Iterate over blocks that are intersected
        for x in intersection_start_index_x..intersection_end_index_x {
            for z in intersection_start_index_z..intersection_end_index_z {
                let tower = &self.data[x][z];
                let blocks = tower.get_layers_in_range(y_low, y_high);
                let x_scale =
                    get_block_portion_in_range(x, intersection_xz.left(), intersection_xz.right());
                let z_scale =
                    get_block_portion_in_range(z, intersection_xz.bottom(), intersection_xz.top());

                for layer in blocks {
                    // let material = layer.0;
                    let y_scale = layer.1;

                    let layer_volume = x_scale * y_scale * z_scale;
                    volume += layer_volume;
                }
            }
        }

        volume
    }

    pub fn get_bounding_rectangle(&self) -> Rectangle {
        let position_in_world = self.position.get_global_position();
        Rectangle::square(position_in_world, minecraft::BLOCKS_IN_CHUNK as f32)
    }

    // TODO: set block? will be a litte complicated since we need to insert a new material stack or
    // extend an existing one
}

pub struct World {
    chunks: [Chunk; WORLD_SIZE * WORLD_SIZE],
    //chunks: [[Chunk; config::WORLD_SIZE]; config::WORLD_SIZE],
    center: ChunkPosition,
    // Init world around a position
    // calculate region and chunk from position
    // this is the position of the CENTER chunk in the world grid
    //
    // calculate position of chunk at grid 0,0
    // iterate over the grid and fill it with chunks
    //
    // each frame update the world grid with new chunks if the center chunk changes
    // - only part of the chunks need to be updated
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
    pub fn new(position: Point3<f32>) -> Self {
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

    pub fn update(&mut self, new_position: Point3<f32>) -> bool {
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

    pub fn get_block(&self, position: Point3<f32>) -> Option<BlockType> {
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

    pub fn sample_volume(&self, kernel: Kernel) -> f32 {
        let kernel_box = kernel.get_bounding_rectangle();
        let y_low = kernel.y_low();
        let y_high = kernel.y_high();

        self.chunks
            .iter()
            .filter_map(|chunk| {
                let chunk_box = chunk.get_bounding_rectangle();
                let Some(intersection) = chunk_box.intersect(kernel_box) else {
                    return None;
                };

                let offset = chunk.position.get_global_position().map(|coord| -coord);
                let intersection_local = intersection.offset_origin(offset);
                let chunk_volume: f32 =
                    chunk.get_chunk_intersection_volume(intersection_local, y_low, y_high);
                Some(chunk_volume)
            })
            .sum()

        //        self.chunks
        //            .iter()
        //            .map(|chunk| chunk.get_chunk_intersection_volume(kernel))
        //            .sum()
    }
}
