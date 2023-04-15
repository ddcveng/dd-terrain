use cgmath::Point3;

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

struct MaterialStack {
    material: BlockType,
    height: u32,
}

// TODO: limit tower height to whatever minecraft allows
struct MaterialTower {
    data: Vec<MaterialStack>,
}

impl MaterialTower {
    pub fn get_block_at_y(&self, y: u32) -> BlockType {
        let mut h = 0;
        for stack in &self.data {
            h += stack.height;
            if h > y {
                return stack.material;
            }
        }

        return BlockType::Air;
    }
}

// A chunks is a 16*y*16 region of blocks
struct Chunk {
    data: [[MaterialStack; 16]; 16],
}

const WORLD_SIZE: usize = 5;
struct World {
    chunks: [[Chunk; WORLD_SIZE]; WORLD_SIZE],
}

impl World {
    pub fn get_block(position: &Point3<f32>) -> BlockType {
        // need either position of camera, or position relative to camera
        //
        // pos / 16 gets us chunk XZ coords
        // pos % 16 gets us block xz coords within the chunks
        // pos.y stays the same and is used to index MaterialTower.get_block_at_y
    }
}

