use std::cmp::min;

use super::material_tower::MaterialStack;
use super::ChunkPosition;
use crate::infrastructure::texture::MaterialBlend;
use crate::minecraft;
use crate::model::common::{get_pallette_texture_coords, is_rigid_block, BlockType};
use crate::model::rectangle::Rectangle;
use crate::model::{Coord, Position, Real};

use array_init::array_init;
use glium::implement_vertex;
use itertools::Itertools;

const EPSILON: Coord = 0.0001;

// Data used for instancing all the blocks
#[derive(Clone, Copy)]
pub struct BlockData {
    pub offset: [f32; 3],
    pub pallette_offset: [f32; 2],
    //instance_color: [f32; 3],
    //height: u32,
    //block_type: u8,
}
implement_vertex!(
    BlockData,
    offset,
    pallette_offset /*, instance_color, height*/
);

const CHUNK_SIZE: usize = minecraft::BLOCKS_IN_CHUNK;

#[derive(Clone, Copy)]
pub struct RigidBlockRecord {
    position: Position,
    material: BlockType,
}

// A chunks is a 16*y*16 region of blocks
pub struct Chunk {
    data: [MaterialStack; CHUNK_SIZE * CHUNK_SIZE],

    rigid_blocks: Vec<RigidBlockRecord>,

    // This is the position of the bottom left corner of the chunk from a top down view
    pub position: ChunkPosition,
}

// TODO: maybe move to common?
fn get_block_coord(world_coord: Coord) -> usize {
    let negative = world_coord < 0.0;

    let coord_positive = if negative { -world_coord } else { world_coord };

    // coord x.yy is still within the block at x -> floor coord_positive
    let block_coord = (coord_positive + EPSILON as Coord) as usize % CHUNK_SIZE;

    if negative {
        CHUNK_SIZE - block_coord - 1
    } else {
        block_coord
    }
}

fn get_block_portion_in_range(block_start: usize, range_start: Coord, range_end: Coord) -> Real {
    let range_start = range_start as Real;
    let range_end = range_end as Real;
    let block_end = (block_start + 1) as Real; // everything is calculated in blocks
    let block_start = block_start as Real;

    let no_overlap = block_end < range_start || block_start > range_end;
    if no_overlap {
        return 0.0;
    }

    let mut portion: Real = 1.0; // whole block is in range

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
            data: array_init(|_inx| MaterialStack::new()),
            rigid_blocks: Vec::new(),
            position: chunk_position,
        }
    }

    fn get_tower(&self, x: usize, z: usize) -> &MaterialStack {
        &self.data[z * CHUNK_SIZE + x]
    }

    fn get_tower_mut(&mut self, x: usize, z: usize) -> &mut MaterialStack {
        &mut self.data[z * CHUNK_SIZE + x]
    }

    // Push block on top of the material tower at x, z
    pub fn push_block(&mut self, x: usize, z: usize, base_height: isize, block: BlockType) {
        let stack = self.get_tower_mut(x, z);
        stack.insert(block, base_height);

        // update rigid block records
        if is_rigid_block(block) {
            let position = Position::new(x as f64, base_height as f64, z as f64);
            let rigid_record = RigidBlockRecord {
                position,
                material: block,
            };

            self.rigid_blocks.push(rigid_record);
        }
    }

    pub fn get_block_data(&self) -> Vec<BlockData> {
        let mut blocks = Vec::<BlockData>::new();
        let (chunk_global_x, chunk_global_z) = self.position.get_global_position_in_chunks();
        let global_offset_blocks_x = chunk_global_x * (minecraft::BLOCKS_IN_CHUNK as i32);
        let global_offset_blocks_z = chunk_global_z * (minecraft::BLOCKS_IN_CHUNK as i32);

        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let stack = self.get_tower(x, z);
                for (y, material) in stack.iter_visible_blocks() {
                    let x_offset_blocks = global_offset_blocks_x + x as i32;
                    let z_offset_blocks = global_offset_blocks_z + z as i32;

                    let block_data = BlockData {
                        offset: [x_offset_blocks as f32, y as f32, z_offset_blocks as f32],
                        pallette_offset: get_pallette_texture_coords(material),
                    };

                    blocks.push(block_data);
                }
            }
        }

        blocks
    }

    pub fn get_rigid_block_data(&self) -> Vec<BlockData> {
        let (chunk_global_x, chunk_global_z) = self.position.get_global_position_in_chunks();
        let global_offset_blocks_x = chunk_global_x * (minecraft::BLOCKS_IN_CHUNK as i32);
        let global_offset_blocks_z = chunk_global_z * (minecraft::BLOCKS_IN_CHUNK as i32);

        self.rigid_blocks
            .iter()
            .map(|rigid_record| {
                let x_offset_blocks = global_offset_blocks_x + rigid_record.position.x as i32;
                let z_offset_blocks = global_offset_blocks_z + rigid_record.position.z as i32;
                BlockData {
                    offset: [
                        x_offset_blocks as f32,
                        rigid_record.position.y as f32,
                        z_offset_blocks as f32,
                    ],
                    pallette_offset: get_pallette_texture_coords(rigid_record.material),
                }
            })
            .collect()
    }

    pub fn get_block_coords(x: Coord, z: Coord) -> (usize, usize) {
        let block_x = get_block_coord(x);
        let block_z = get_block_coord(z);

        (block_x, block_z)
    }

    pub fn get_block(&self, x: usize, y: isize, z: usize) -> BlockType {
        let tower = self.get_tower(x, z);

        tower.get_block_at_y(y)
    }

    pub fn enumerate_blocks(
        &self,
        y_low: isize,
        y_high: isize,
    ) -> impl Iterator<Item = (Position, BlockType)> + '_ {
        self.data.iter().enumerate().flat_map(move |(i, tower)| {
            let block_x = (i % CHUNK_SIZE) as Coord;
            let block_z = (i / CHUNK_SIZE) as Coord;

            tower
                .iter_visible_blocks()
                .filter(move |(block_y, _material)| *block_y >= y_low && *block_y < y_high)
                .map(move |(block_y, material)| {
                    let position = Position::new(block_x, block_y as Coord, block_z);

                    (position, material)
                })
        })
    }

    // Intersection is a rectangle local to the chunk - its origin is in chunk local coordinates
    // and the whole rectangle fits inside the chunk
    pub fn get_chunk_intersection_volume(
        &self,
        intersection_xz: Rectangle,
        y_low: Coord,
        y_high: Coord,
    ) -> Real {
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

        // Iterate over blocks that are intersected
        let intersection_range = (intersection_start_index_x..intersection_end_index_x)
            .cartesian_product(intersection_start_index_z..intersection_end_index_z);
        let volume = intersection_range.fold(0.0, move |acc, (x, z)| {
            let x_scale =
                get_block_portion_in_range(x, intersection_xz.left(), intersection_xz.right());
            let z_scale =
                get_block_portion_in_range(z, intersection_xz.bottom(), intersection_xz.top());
            let y_scale = self.get_tower(x, z).get_intersection_size(y_low, y_high);

            let intersection_volume = x_scale * y_scale * z_scale;
            acc + intersection_volume
        });

        volume
    }

    pub fn get_material_blend(
        &self,
        intersection_xz: Rectangle,
        y_low: Coord,
        y_high: Coord,
    ) -> MaterialBlend {
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

        // Iterate over blocks that are intersected
        let intersection_range = (intersection_start_index_x..intersection_end_index_x)
            .cartesian_product(intersection_start_index_z..intersection_end_index_z);

        let volume = intersection_range.fold(MaterialBlend::new(), move |mut blend, (x, z)| {
            let x_scale =
                get_block_portion_in_range(x, intersection_xz.left(), intersection_xz.right());
            let z_scale =
                get_block_portion_in_range(z, intersection_xz.bottom(), intersection_xz.top());

            let tower = self.get_tower(x, z);
            for (y_scale, material) in tower.iter_intersecting_blocks(y_low, y_high) {
                let block_intersection_size = x_scale * y_scale * z_scale;
                blend.mix(material, block_intersection_size);
            }

            blend
        });

        volume
    }

    pub fn get_bounding_rectangle(&self) -> Rectangle {
        let position_in_world = self.position.get_global_position();
        Rectangle::square(position_in_world, CHUNK_SIZE as Coord)
    }
}
