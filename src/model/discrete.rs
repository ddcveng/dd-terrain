use crate::config;
use crate::config::WORLD_SIZE;
use crate::get_minecraft_chunk_position;
use crate::minecraft;
use array_init::array_init;
use cgmath::Point3;
use glium::implement_vertex;
use itertools;

// TODO: is 1 byte for block type enough?
#[derive(Clone, Copy)]
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
    //block_type: u8,
}
implement_vertex!(BlockData, offset, instance_color);

#[derive(Clone, Copy)]
struct MaterialStack {
    material: BlockType,
    height: usize,
    base_height: isize,
}

// TODO: limit tower height to whatever minecraft allows
struct MaterialTower {
    pub data: Vec<MaterialStack>,
}

impl MaterialTower {
    pub fn new() -> Self {
        MaterialTower { data: Vec::new() }
    }

    pub fn get_block_at_y(&self, y: usize) -> BlockType {
        let mut h: usize = 0;
        for stack in &self.data {
            h += stack.height;
            if h > y {
                return stack.material;
            }
        }

        // If there is no block recorded at this height, assume its air
        // note that Air blocks can be recorded in the stack so the loop above can also return Air
        return BlockType::Air;
    }

    pub fn push(&mut self, block: BlockType, base_height: isize) {
        // TODO: if there is already a segment for the same block, just increase its height, do not
        // add a new segment
        let segment = MaterialStack {
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

// A chunks is a 16*y*16 region of blocks
pub struct Chunk {
    // A column major grid of towers
    data: [[MaterialTower; minecraft::BLOCKS_IN_CHUNK]; minecraft::BLOCKS_IN_CHUNK],
    position: ChunkPosition,
}

impl Chunk {
    pub fn new(chunk_position: ChunkPosition) -> Self {
        Chunk {
            data: array_init(|_x| array_init(|_y| MaterialTower::new())),
            position: chunk_position,
        }
    }

    pub fn get_block(&self, x: usize, y: usize, z: usize) -> BlockType {
        let stack = &self.data[x][z];
        stack.get_block_at_y(y)
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
                    };
                    blocks.push(block_data);
                }
                z += 1;
            }
            x += 1;
        }

        blocks
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
        //println!("center chunk position: {:?}", center_chunk_position);
        // Get position of chunk that corresponds to 0,0 in the world grid
        let base_chunk_position = center_chunk_position
            .offset(-(OFFSET_FROM_CENTER as i32), -(OFFSET_FROM_CENTER as i32));
        //println!("base chunk position: {:?}", base_chunk_position);

        World {
            chunks: array_init(|index| {
                let x = index % config::WORLD_SIZE;
                let z = index / config::WORLD_SIZE;
                let chunk_position = base_chunk_position.offset(x as i32, z as i32);

                minecraft::get_chunk(chunk_position)
            }),
            //            chunks: array_init(|offset_x| {
            //                array_init(|offset_z| {
            //                    let chunk_position =
            //                        base_chunk_position.offset(offset_x as i32, offset_z as i32);
            //                    minecraft::get_chunk(chunk_position)
            //                })
            //            }),
            center: center_chunk_position,
        }
    }

    pub fn get_block_data(&self) -> Vec<BlockData> {
        let mut blocks = Vec::<BlockData>::new();

        // TODO: move by 16 * chunk offset
        // chunk offset ranges from -OFFSETFROMCENTER to OFFSETFROMCENTER
        for (i, chunk) in self.chunks.iter().enumerate() {
            //println!("CHUNK OFFSET: {} {}", offset_x, offset_z);
            let mut chunk_blocks = chunk.get_block_data();
            if i >= 4 * WORLD_SIZE {
                println!(
                    "BLOCKS IN CHUNK {}; OFFSET {:?}",
                    chunk_blocks.len(),
                    chunk.position
                );
            }

            blocks.append(&mut chunk_blocks);
        }

        blocks
    }

    pub fn update(&mut self, new_position: Point3<f32>) -> bool {
        let new_center_chunk = get_minecraft_chunk_position(new_position);
        if self.center == new_center_chunk {
            return false;
        }

        println!(
            "old center: {:?} new center: {:?}",
            self.center, new_center_chunk
        );
        // Get the direction of change
        let (diff_x, diff_z) = get_difference(&self.center, &new_center_chunk);
        self.center = new_center_chunk;
        println!("CHUNK DIFF: {} {}", diff_x, diff_z);

        // TODO: need to find out the relationship between world grid x,z and chunk x,z, it should
        // work the same. also index of block in chunk should work the same
        // Update the world matrix by either shifting chunks based on the direction, or loading
        // needed chunks
        let reverse_x = diff_x < 0;
        let reverse_z = diff_z < 0;

        // last_z should start as first value from iterator, which can be either 0 oir WORLD_SIZE
        let mut last_z = 0;
        for z in get_iterator(0, WORLD_SIZE, reverse_z) {
            let mut last_x = 0;
            for x in get_iterator(0, WORLD_SIZE, reverse_x) {
                //println!("EVALUATING CELL {} {}", x, z);
                let next_x = x as i32 + diff_x;
                let next_z = z as i32 + diff_z;
                if let Some(next_x) = clamp_chunk_index(next_x) {
                    if let Some(next_z) = clamp_chunk_index(next_z) {
                        let current_index = z * config::WORLD_SIZE + x;
                        let next_index = next_z * config::WORLD_SIZE + next_x;

                        //println!("SWAPPING CHUNKS {}<>{}", current_index, next_index);
                        self.chunks.swap(current_index, next_index);
                        continue;
                    }
                }

                let current_position = &self.chunks[last_z * config::WORLD_SIZE + last_x].position;
                let position_to_load = current_position.offset(diff_x, diff_z);
                println!(
                    "LOADING NEW CHUNK {:?} as ARRAY ELEMENT {}",
                    position_to_load,
                    z * WORLD_SIZE + x
                );
                println!("ORIGINAL CHUNK POSITION {:?}", current_position);

                let new_chunk = minecraft::get_chunk(position_to_load);
                //println!("NEW CHUNK POSITION {:?}", new_chunk.position);
                self.chunks[z * config::WORLD_SIZE + x] = new_chunk;
                last_x = x;
            }
            last_z = z;
        }

        for (i, chunk) in self.chunks.iter().enumerate() {
            let x = i % WORLD_SIZE;
            let z = i / WORLD_SIZE;
            println!("[{}, {}] - {:?}", x, z, chunk.position);
        }

        return true;
    }
}
