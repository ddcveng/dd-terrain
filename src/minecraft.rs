use fastanvil::{CurrentJavaChunk, Region};
use fastnbt::from_bytes;
use glium::implement_vertex;

#[derive(Clone, Copy)]
pub struct BlockData {
    offset: [f32; 3],
}
implement_vertex!(BlockData, offset);

const WORLD_FILE: &str = r#"C:\Users\edo15\AppData\Roaming\.minecraft\saves\bananko\region\r.0.0.mca"#;
const AIR_BLOCK_ID: &str = "minecraft:air";

pub fn get_chunk() -> Vec<BlockData> {
    let file = std::fs::File::open(WORLD_FILE).unwrap();

    let mut region = Region::from_stream(file).unwrap();
    let data = region.read_chunk(6, 2).unwrap().unwrap();

    let mut blocks_in_chunk = Vec::new();
    let chunk: CurrentJavaChunk = from_bytes(data.as_slice()).unwrap();
    if let Some(tower) = chunk.sections {
        let y = 112;

        if let Some(section) = tower.get_section_for_y(y) {
            let block_states = &section.block_states;
            let palette = block_states.palette();
            for (i, palette_index) in block_states.try_iter_indices().unwrap().enumerate() {
                let x = i & 0x000F;
                let y = (i & 0x0F00) >> 8;
                let z = (i & 0x00F0) >> 4;

                let block = &palette[palette_index];
                //println!("{}", block.encoded_description());
                if block.name() != AIR_BLOCK_ID {
                   blocks_in_chunk.push(BlockData { offset: [x as f32, y as f32, z as f32] }); 
                }
            }
        }
    }

    blocks_in_chunk
}
