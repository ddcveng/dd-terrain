// TODO: is 1 byte for block type enough?
// Note: Unknown must always be the last variant,
// or at least the variant with the largest value.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum BlockType {
    Air = 0,
    Dirt = 1,
    Grass = 2,
    Stone = 3,
    Wood = 4,
    Leaves = 5,
    Sand = 6,
    Ore = 7,
    Unknown = 8,
}

pub const BLOCK_TYPES: usize = (BlockType::Unknown as usize) + 1;

const PALLETTE_SIZE: usize = 4;
pub const BLOCK_TEXTURE_FRACTION: f32 = 1.0 / (PALLETTE_SIZE as f32);
pub fn get_pallette_texture_coords(block_type: BlockType) -> [f32; 2] {
    let (x_offset, y_offset) = match block_type {
        BlockType::Grass => (0, 3),
        BlockType::Dirt => (0, 2),
        BlockType::Stone => (1, 3),
        BlockType::Sand => (2, 3),
        BlockType::Ore => (3, 3),
        _ => (3, 0),
    };

    [
        (x_offset as f32) * BLOCK_TEXTURE_FRACTION,
        (y_offset as f32) * BLOCK_TEXTURE_FRACTION,
    ]
}

pub fn get_block_color(block_type: BlockType) -> [f32; 3] {
    match block_type {
        BlockType::Grass => [0.09, 0.4, 0.05],
        BlockType::Dirt => [0.36, 0.09, 0.05],
        BlockType::Stone => [0.6, 0.6, 0.6],
        BlockType::Sand => [0.76, 0.69, 0.5],
        BlockType::Ore => [0.2, 0.2, 0.2],
        _ => [1.0, 0.0, 0.0],
    }
}

pub fn is_visible_block(material: BlockType) -> bool {
    !matches!(material, BlockType::Air)
}

const RIGID_MATERIALS: [BlockType; 2] = [BlockType::Wood, BlockType::Leaves];
pub fn is_rigid_block(material: BlockType) -> bool {
    RIGID_MATERIALS.contains(&material)
}
