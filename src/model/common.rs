use std::collections::HashSet;

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

enum MaterialOperation {
    Include,
    Exclude,
}

// TODO: performance
pub struct MaterialSetup {
    smoothable_materials: HashSet<BlockType>,
    rigid_materials: HashSet<BlockType>,
    op: MaterialOperation,
}

impl MaterialSetup {
    pub fn include<const S: usize, const R: usize>(
        included: [BlockType; S],
        rigid: [BlockType; R],
    ) -> Self {
        MaterialSetup {
            smoothable_materials: HashSet::from(included),
            rigid_materials: HashSet::from(rigid),
            op: MaterialOperation::Include,
        }
    }

    pub fn exclude<const S: usize, const R: usize>(
        excluded: [BlockType; S],
        rigid: [BlockType; R],
    ) -> Self {
        MaterialSetup {
            smoothable_materials: HashSet::from(excluded),
            rigid_materials: HashSet::from(rigid),
            op: MaterialOperation::Exclude,
        }
    }

    pub fn all_smooth<const R: usize>(rigid: [BlockType; R]) -> Self {
        MaterialSetup {
            smoothable_materials: HashSet::new(),
            rigid_materials: HashSet::from(rigid),
            op: MaterialOperation::Exclude,
        }
    }

    pub fn is_material_smoothable(&self, material: BlockType) -> bool {
        let possibly_smoothable =
            !matches!(material, BlockType::Air) && !self.rigid_materials.contains(&material);

        if !possibly_smoothable {
            return false;
        }

        let is_included = match self.op {
            MaterialOperation::Include => self.smoothable_materials.contains(&material),
            MaterialOperation::Exclude => !self.smoothable_materials.contains(&material),
        };

        return is_included;
    }

    pub fn contributes_color(&self, material: BlockType) -> bool {
        let is_included = match self.op {
            MaterialOperation::Include => self.smoothable_materials.contains(&material),
            MaterialOperation::Exclude => !self.smoothable_materials.contains(&material),
        };

        is_included || self.is_rigid(material)
    }

    pub fn is_rigid(&self, material: BlockType) -> bool {
        return self.rigid_materials.contains(&material);
    }

    pub fn no_rigid(&self) -> bool {
        self.rigid_materials.is_empty()
    }
}

pub const RIGID_MATERIALS: [BlockType; 5] = [
    BlockType::Wood,
    BlockType::Cobblestone,
    BlockType::Planks,
    BlockType::Glass,
    BlockType::Unknown,
];

//const RIGID_MATERIALS: u16 = 0b1110010000010000;
pub fn is_rigid_block(material: BlockType) -> bool {
    //let material_index = material as u16;

    //let bit_value = 1 << material_index;
    //(RIGID_MATERIALS & bit_value) != 0
    matches!(material, BlockType::Wood)
    //RIGID_MATERIALS.contains(&material)
}
