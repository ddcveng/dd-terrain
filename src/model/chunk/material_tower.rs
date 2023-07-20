use crate::model::{
    common::{is_visible_block, BlockType, MaterialSetup},
    Coord, Real,
};

const BLOCK_SIZE: Real = 1.0;
const STACK_HEIGHT: usize = 384;
const NEGATIVE_HEIGHT_PART: isize = 64;

// Contains blocks from y = -64 to y = 320 in ascending order
pub struct MaterialStack {
    blocks: Vec<BlockType>,
}

fn index_to_height(index: usize) -> isize {
    (index as isize) - NEGATIVE_HEIGHT_PART
}

fn height_to_index(height: isize) -> usize {
    (height + NEGATIVE_HEIGHT_PART) as usize
}

impl MaterialStack {
    pub fn new() -> Self {
        let mut data: Vec<BlockType> = Vec::with_capacity(STACK_HEIGHT);
        data.resize(STACK_HEIGHT, BlockType::Air);

        MaterialStack { blocks: data }
    }

    pub fn insert(&mut self, material: BlockType, base_height: isize) {
        let stack_index = height_to_index(base_height);
        //println!("height: {base_height} -> index: {stack_index}");
        self.blocks[stack_index] = material;
    }

    pub fn get_intersection_size(
        &self,
        y_low: Coord,
        y_high: Coord,
        material_setup: &MaterialSetup,
    ) -> Real {
        let low_floor = y_low.floor();
        let high_ceil = y_high.ceil();
        let low_index = height_to_index(low_floor as isize);
        let high_index = height_to_index(high_ceil as isize);

        let blocks_in_range = (low_index..high_index)
            .map(|i| self.blocks[i])
            .filter(|material| material_setup.is_material_smoothable(*material))
            .count();

        if blocks_in_range == 0 {
            return 0.0;
        }

        let excess_low = {
            let cutoff = material_setup.is_material_smoothable(self.blocks[low_index]);
            //let cutoff = !rigid_set.contains(&self.blocks[low_index]); //is_smoothable_block(self.blocks[low_index]);
            match cutoff {
                true => (y_low - low_floor) as Real,
                false => 0.0,
            }
        };
        let excess_high = {
            let cutoff = material_setup.is_material_smoothable(self.blocks[high_index - 1]);
            //let cutoff = !rigid_set.contains(&self.blocks[high_index - 1]);
            match cutoff {
                true => (high_ceil - y_high) as Real,
                false => 0.0,
            }
        };

        let intersection_size = (blocks_in_range as Real) - excess_low - excess_high;

        assert!(intersection_size > 0.0);
        intersection_size
    }

    pub fn iter_intersecting_blocks(
        &self,
        y_low: Coord,
        y_high: Coord,
    ) -> impl Iterator<Item = (Real, BlockType)> + '_ {
        let low_floor = y_low.floor();
        let high_ceil = y_high.ceil();
        let low_index = height_to_index(low_floor as isize);
        let high_index = height_to_index(high_ceil as isize);

        let intersecting_blocks = (low_index..high_index)
            .map(|i| (index_to_height(i), self.blocks[i].clone()))
            .filter(|(_, material)| is_visible_block(*material));

        let blocks_with_intersection_size =
            intersecting_blocks.map(move |(base_height, material)| {
                let base_height = base_height as Coord;
                let lower_cutoff = if base_height < y_low {
                    y_low - base_height
                } else {
                    0.0
                };

                let upper_cutoff = if y_high - base_height < BLOCK_SIZE {
                    y_high - base_height
                } else {
                    0.0
                };

                let intersection_size = BLOCK_SIZE - lower_cutoff - upper_cutoff;

                (intersection_size, material)
            });

        blocks_with_intersection_size
    }

    pub fn iter_visible_blocks(&self) -> impl Iterator<Item = (isize, BlockType)> + '_ {
        self.blocks
            .iter()
            .enumerate()
            .filter(|(_i, material)| is_visible_block(**material))
            .map(|(i, material)| (index_to_height(i), material.clone()))
    }

    pub fn iter_blocks_in_range(
        &self,
        y_low: Coord,
        y_high: Coord,
    ) -> impl Iterator<Item = (isize, BlockType)> + '_ {
        let low_floor = y_low.floor();
        let high_ceil = y_high.ceil();
        let low_index = height_to_index(low_floor as isize);
        let high_index = height_to_index(high_ceil as isize);

        let intersecting_blocks =
            (low_index..high_index).map(|i| (index_to_height(i), self.blocks[i].clone()));

        intersecting_blocks
    }

    pub fn get_block_at_y(&self, y: isize) -> BlockType {
        let block_index = height_to_index(y);
        self.blocks[block_index]
    }
}
