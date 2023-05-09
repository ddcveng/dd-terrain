use crate::minecraft;
use cgmath::Point2;

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

    pub fn get_global_position(&self) -> Point2<f32> {
        let (chunk_x, chunk_z) = self.get_global_position_in_chunks();
        Point2::new(
            (chunk_x * minecraft::BLOCKS_IN_CHUNK as i32) as f32,
            (chunk_z * minecraft::BLOCKS_IN_CHUNK as i32) as f32,
        )
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
