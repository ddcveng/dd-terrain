use std::cmp::min;

use super::material_tower::MaterialTower;
use super::ChunkPosition;
use crate::minecraft;
use crate::model::common::{get_block_color, BlockType};
use crate::model::rectangle::Rectangle;
use array_init::array_init;
use glium::implement_vertex;

const EPSILON: f32 = 0.0001;

// Data used for instancing all the blocks
#[derive(Clone, Copy)]
pub struct BlockData {
    offset: [f32; 3],
    instance_color: [f32; 3],
    height: u32,
    //block_type: u8,
}
implement_vertex!(BlockData, offset, instance_color, height);

const CHUNK_SIZE: usize = minecraft::BLOCKS_IN_CHUNK;

// A chunks is a 16*y*16 region of blocks
pub struct Chunk {
    data: [MaterialTower; CHUNK_SIZE * CHUNK_SIZE],

    // This is the position of the bottom left corner of the chunk from a top down view
    pub position: ChunkPosition,
}

// TODO: maybe move to common?
fn get_block_coord(world_coord: f32) -> usize {
    let negative = world_coord < 0.0;

    let coord_positive = if negative { -world_coord } else { world_coord };

    // coord x.yy is still within the block at x -> floor coord_positive
    let block_coord = (coord_positive + EPSILON) as usize % CHUNK_SIZE;

    if negative {
        CHUNK_SIZE - block_coord - 1
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

impl Chunk {
    pub fn new(chunk_position: ChunkPosition) -> Self {
        Chunk {
            data: array_init(|_inx| MaterialTower::new()),
            position: chunk_position,
        }
    }

    fn get_tower(&self, x: usize, z: usize) -> &MaterialTower {
        &self.data[z * CHUNK_SIZE + x]
    }

    fn get_tower_mut(&mut self, x: usize, z: usize) -> &mut MaterialTower {
        &mut self.data[z * CHUNK_SIZE + x]
    }

    // Push block on top of the material tower at x, z
    pub fn push_block(&mut self, x: usize, z: usize, base_height: isize, block: BlockType) {
        let tower = self.get_tower_mut(x, z);
        tower.push(block, base_height);
    }

    pub fn get_block_data(&self) -> Vec<BlockData> {
        let mut blocks = Vec::<BlockData>::new();
        let (chunk_global_x, chunk_global_z) = self.position.get_global_position_in_chunks();
        let global_offset_blocks_x = chunk_global_x * (minecraft::BLOCKS_IN_CHUNK as i32);
        let global_offset_blocks_z = chunk_global_z * (minecraft::BLOCKS_IN_CHUNK as i32);

        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let tower = self.get_tower(x, z);
                for segment in &tower.data {
                    let x_offset_blocks = global_offset_blocks_x + x as i32;
                    let z_offset_blocks = global_offset_blocks_z + z as i32;
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
            }
        }

        blocks
    }

    pub fn get_block_coords(x: f32, z: f32) -> (usize, usize) {
        let block_x = get_block_coord(x);
        let block_z = get_block_coord(z);

        (block_x, block_z)
    }

    pub fn get_block(&self, x: usize, y: isize, z: usize) -> BlockType {
        let tower = self.get_tower(x, z);

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
                let tower = self.get_tower(x, z);
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
        Rectangle::square(position_in_world, CHUNK_SIZE as f32)
    }
}
