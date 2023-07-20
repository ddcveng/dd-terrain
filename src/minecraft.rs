use fastanvil::{CurrentJavaChunk, Region};
use fastnbt::from_bytes;
use std::path::Path;

use crate::config;
use crate::model::chunk::ChunkPosition;
use crate::model::common::BlockType;
use crate::model::Position;

pub const MIN_BLOCK_Y: isize = -64; // TODO: Real value is -64
pub const MAX_BLOCK_Y: isize = 320;

// These are 1D, the actual number should be this squared but that isnt very useful
pub const CHUNKS_IN_REGION: usize = 32;
pub const BLOCKS_IN_CHUNK: usize = 16;

// The name of the block has to match the string exactly
const BLOCK_MAP_EXACT: [(&str, BlockType); 12] = [
    ("grass_block", BlockType::Grass),
    ("air", BlockType::Air),
    ("granite", BlockType::Dirt),
    ("diorite", BlockType::DarkStone),
    ("stone", BlockType::Stone),
    ("gravel", BlockType::Stone),
    ("andesite", BlockType::DarkStone),
    ("deepslate", BlockType::Stone),
    ("red_sand", BlockType::RedSand),
    ("cobblestone", BlockType::Cobblestone),
    ("oak_stairs", BlockType::Planks),
    ("white_terracotta", BlockType::Cobblestone),
];

// The name of the block has to contain the key to match
// Note that order matters as some blocks may match multiple keys.
const BLOCK_MAP_NONSPECIFIC: [(&str, BlockType); 10] = [
    ("dirt", BlockType::Dirt),
    ("stone", BlockType::Cobblestone),
    ("sand", BlockType::Sand),
    ("ore", BlockType::Ore),
    ("log", BlockType::Wood),
    ("leaves", BlockType::Leaves),
    ("water", BlockType::Water),
    ("lava", BlockType::Lava),
    ("plank", BlockType::Planks),
    ("glass", BlockType::Glass),
];

const BLOCK_BLACKLIST: [&str; 11] = [
    "dead_bush",
    "grass",
    "fern",
    "tall_grass",
    "vine",
    "cocoa",
    "poppy",
    "dandelion",
    "torch",
    "wall_torch",
    "oak_door",
];

// Alias type definition to avoid ambiguity with fastanvil::Chunk
type DDChunk = crate::model::chunk::Chunk;

pub fn get_chunk(/*region_loader: &RegionFileLoader,*/ chunk_position: ChunkPosition,) -> DDChunk {
    let mut dd_chunk = DDChunk::new(chunk_position);

    let region_file_path = build_region_filepath(chunk_position.region_x, chunk_position.region_z);
    let file = std::fs::File::open(region_file_path).unwrap();
    let mut region = Region::from_stream(file).unwrap();

    let data = match region.read_chunk(chunk_position.chunk_x, chunk_position.chunk_z) {
        Ok(opt_data) => match opt_data {
            Some(chunk_data) => chunk_data,
            None => {
                println!(
                    "INFO: chunk at position {:?} was not yet generated",
                    chunk_position
                );
                return dd_chunk;
            }
        },
        Err(e) => {
            eprintln!("Failed to load chunk data from region - {}", e);
            return dd_chunk;
        }
    };

    let chunk: CurrentJavaChunk = from_bytes(data.as_slice()).unwrap();

    if let Some(tower) = chunk.sections {
        for section in tower.sections() {
            let section_base_y = section.y as isize * 16;

            // TODO: "since 1.17 depth of worlds can be customized" - is this relevant?

            let block_states = &section.block_states;
            let blocks_iterator = match block_states.try_iter_indices() {
                Some(iterator) => iterator,
                None => continue, // None means there are no blocks, i.e. section of pure air
            };

            let palette = block_states.palette();
            for (i, palette_index) in blocks_iterator.enumerate() {
                let block = &palette[palette_index];
                let Some(block_id) = block.name().strip_prefix("minecraft:") else {
                    continue;
                };

                let block_type = get_block_type_ng(block_id);

                match block_type {
                    BlockType::Unknown => {
                        // TODO: this is bad
                        if BLOCK_BLACKLIST.contains(&block_id) {
                            continue;
                        }
                    }
                    BlockType::Air => continue,
                    _ => (),
                };

                let x = i & 0x000F;
                let y = (i & 0x0F00) >> 8;
                let z = (i & 0x00F0) >> 4;

                dd_chunk.push_block(x, z, section_base_y + y as isize, block_type);
            }
        }
    }

    dd_chunk
}

fn build_region_filepath(region_x: i32, region_z: i32) -> String {
    let region_file_name = format!("r.{}.{}.mca", region_x, region_z);
    let region_file_path = Path::new(config::WORLD_FOLDER)
        //.join("region")
        .join(region_file_name);

    region_file_path.to_str().unwrap().to_owned()
}

fn get_block_type_ng(block_id: &str) -> BlockType {
    let exact_match = BLOCK_MAP_EXACT.iter().find(|(key, _)| block_id == *key);
    if let Some((_, block_type)) = exact_match {
        return *block_type;
    }

    let partial_match = BLOCK_MAP_NONSPECIFIC
        .iter()
        .find(|(key, _)| block_id.contains(key));
    if let Some((_, block_type)) = partial_match {
        return *block_type;
    }

    BlockType::Unknown
}

// negative regions are indexed shifted by 1 to differentiate between positive and negative zeros
pub fn get_minecraft_chunk_position(world_position: Position) -> ChunkPosition {
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
