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

pub fn get_block_color(block_type: BlockType) -> [f32; 3] {
    match block_type {
        BlockType::Grass => [0.09, 0.4, 0.05],
        BlockType::Dirt => [0.36, 0.09, 0.05],
        BlockType::Stone => [0.6, 0.6, 0.6],
        BlockType::Sand => [0.76, 0.69, 0.5],
        _ => [1.0, 0.0, 0.0],
    }
}
