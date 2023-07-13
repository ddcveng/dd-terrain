use super::Real;

// Note: Unknown must always be the last variant,
// or at least the variant with the largest value.
//
// The integer values are used in shaders to determine the texture,
// changing them requires updating the shaders
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum BlockType {
    Air = 0,
    Dirt = 1,
    Grass = 2,
    Stone = 3,
    Wood = 4,
    Leaves = 5,
    Sand = 6,
    Ore = 7,
    Water = 8,
    Lava = 9,
    Planks = 10,
    DarkStone = 11,
    RedSand = 12,
    Cobblestone = 13,
    Glass = 14,
    Unknown = 15,
}

impl TryFrom<usize> for BlockType {
    type Error = ();

    fn try_from(v: usize) -> Result<Self, Self::Error> {
        match v {
            x if x == BlockType::Dirt as usize => Ok(BlockType::Dirt),
            x if x == BlockType::Grass as usize => Ok(BlockType::Grass),
            x if x == BlockType::Stone as usize => Ok(BlockType::Stone),
            x if x == BlockType::Wood as usize => Ok(BlockType::Wood),
            x if x == BlockType::Leaves as usize => Ok(BlockType::Leaves),
            x if x == BlockType::Sand as usize => Ok(BlockType::Sand),
            x if x == BlockType::Ore as usize => Ok(BlockType::Ore),
            x if x == BlockType::Water as usize => Ok(BlockType::Water),
            x if x == BlockType::Lava as usize => Ok(BlockType::Lava),
            x if x == BlockType::Planks as usize => Ok(BlockType::Planks),
            x if x == BlockType::DarkStone as usize => Ok(BlockType::DarkStone),
            x if x == BlockType::RedSand as usize => Ok(BlockType::RedSand),
            x if x == BlockType::Cobblestone as usize => Ok(BlockType::Cobblestone),
            x if x == BlockType::Glass as usize => Ok(BlockType::Glass),
            _ => Err(()),
        }
    }
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
        BlockType::Wood => (1, 2),
        BlockType::Leaves => (2, 2),
        BlockType::Water => (3, 2),
        BlockType::Lava => (0, 1),
        BlockType::Planks => (1, 1),
        BlockType::DarkStone => (2, 1),
        BlockType::RedSand => (3, 1),
        BlockType::Cobblestone => (0, 0),
        BlockType::Glass => (1, 0),
        _ => (3, 0),
    };

    [
        (x_offset as f32) * BLOCK_TEXTURE_FRACTION,
        (y_offset as f32) * BLOCK_TEXTURE_FRACTION,
    ]
}

pub fn activation_treshold(block_type: BlockType) -> Real {
    match block_type {
        BlockType::Dirt => 0.45,
        BlockType::Leaves => 0.9,
        _ => 0.0, // always activate
    }
}

pub fn is_visible_block(material: BlockType) -> bool {
    !matches!(material, BlockType::Air)
}

const RIGID_MATERIALS: [BlockType; 5] = [
    BlockType::Wood,
    BlockType::Cobblestone,
    BlockType::Planks,
    BlockType::Glass,
    BlockType::Unknown,
];
pub fn is_rigid_block(material: BlockType) -> bool {
    RIGID_MATERIALS.contains(&material)
}
