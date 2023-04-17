use cgmath::Point3;
use fastanvil::{CurrentJavaChunk, Region};
use fastnbt::from_bytes;
use std::path::Path;

use crate::config;
use crate::model::discrete::{BlockType, Chunk, ChunkPosition};

pub const MIN_BLOCK_Y: isize = 32; // TODO: Real value is -64
const MAX_BLOCK_Y: isize = 320;
const SECTION_HEIGHT: usize = 16;
const BLOCK_BLACKLIST: [&str; 4] = [
    "minecraft:dead_bush",
    "minecraft:grass",
    "minecraft:fern",
    "minecraft:tall_grass",
];

// These are 1D, the actual number should be this squared but that isnt very useful
pub const CHUNKS_IN_REGION: usize = 32;
pub const BLOCKS_IN_CHUNK: usize = 16;

pub fn get_chunk(chunk_position: ChunkPosition) -> Chunk {
    let region_file_path = build_region_filepath(chunk_position.region_x, chunk_position.region_z);
    let file = std::fs::File::open(region_file_path).unwrap();

    let mut region = Region::from_stream(file).unwrap();
    let data = region
        .read_chunk(chunk_position.chunk_x, chunk_position.chunk_z)
        .unwrap()
        .unwrap();

    let mut dd_chunk = Chunk::new(chunk_position);
    let chunk: CurrentJavaChunk = from_bytes(data.as_slice()).unwrap();
    if let Some(tower) = chunk.sections {
        // TODO: "since 1.17 depth of  worlds can be customized" - is this relevant?
        for section_base_y in (MIN_BLOCK_Y..MAX_BLOCK_Y).step_by(SECTION_HEIGHT) {
            let Some(section) = tower.get_section_for_y(section_base_y) else { continue };

            let block_states = &section.block_states;
            let blocks_iterator = match block_states.try_iter_indices() {
                Some(iterator) => iterator,
                None => continue, // None means there are no blocks, i.e. section of pure air
            };

            let palette = block_states.palette();
            for (i, palette_index) in blocks_iterator.enumerate() {
                let x = i & 0x000F;
                let y = (i & 0x0F00) >> 8;
                let z = (i & 0x00F0) >> 4;

                let block = &palette[palette_index];
                let block_type = get_block_type(block.name());

                match block_type {
                    BlockType::Unknown => {
                        // TODO: this is bad
                        if BLOCK_BLACKLIST.contains(&block.name()) {
                            println!("blocked");
                            continue;
                        }
                    }
                    BlockType::Air => continue,
                    _ => (),
                };

                dd_chunk.push_block(x, z, section_base_y + y as isize, block_type);
            }
        }
    }

    dd_chunk
}

fn build_region_filepath(region_x: i32, region_z: i32) -> String {
    let region_file_name = format!("r.{}.{}.mca", region_x, region_z);
    let region_file_path = Path::new(config::WORLD_FOLDER)
        .join("region")
        .join(region_file_name);

    region_file_path.to_str().unwrap().to_owned()
}

pub fn get_block_type(block_id: &str) -> BlockType {
    match block_id {
        "minecraft:dirt" => BlockType::Dirt,
        "minecraft:stone" => BlockType::Stone,
        "minecraft:grass_block" => BlockType::Grass,
        "minecraft:air" => BlockType::Air,
        _ => BlockType::Unknown,
    }
}

pub fn get_minecraft_chunk_position(world_position: Point3<f32>) -> ChunkPosition {
    // negative regions are indexed shifted by 1 to differentiate between positive and negative
    // zeros
    let (region_x, region_z): (i32, i32) = {
        let mut bias = if world_position.x < 0.0 { -1 } else { 0 };
        let x = (world_position.x / (32.0 * 16.0)) as i32 + bias;

        bias = if world_position.z < 0.0 { -1 } else { 0 };
        let z = (world_position.z / (32.0 * 16.0)) as i32 + bias;

        (x, z)
    };
    let (chunk_x, chunk_z): (usize, usize) = {
        let x_within_region = ((world_position.x / 16.0).abs() as usize) % 32;
        let z_within_region = ((world_position.z / 16.0).abs() as usize) % 32;

        let x = if region_x < 0 {
            31 - x_within_region
        } else {
            x_within_region
        };

        let z = if region_z < 0 {
            31 - z_within_region
        } else {
            z_within_region
        };

        (x, z)
    };

    ChunkPosition {
        region_x,
        region_z,
        chunk_x,
        chunk_z,
    }
}
