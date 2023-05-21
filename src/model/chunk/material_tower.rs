use crate::model::{common::BlockType, Coord, Real};

#[derive(Clone, Copy)]
pub struct MaterialLayer {
    pub material: BlockType,
    pub height: u32,
    pub base_height: isize,
}

// Stores continuous layers of blocks,
// there can be gaps of air between layers, these are not stored.
//
// Block layers are ordered from lowest to highest y coordinate
pub struct MaterialTower {
    pub data: Vec<MaterialLayer>,
    lower_bound: Option<isize>,
    upper_bound: Option<isize>,
}

impl MaterialTower {
    pub fn new() -> Self {
        MaterialTower {
            data: Vec::new(),
            lower_bound: None,
            upper_bound: None,
        }
    }

    pub fn get_block_at_y(&self, y: isize) -> BlockType {
        let layer = self.data.iter().find(|layer| {
            y >= layer.base_height && ((y - layer.base_height) as u32) < layer.height
        });

        if let Some(layer) = layer {
            return layer.material;
        }

        // If there is no block recorded at this height, assume its air
        return BlockType::Air;
    }

    pub fn get_size_of_blocks_in_range(&self, y_low: Coord, y_high: Coord) -> Real {
        let Some(lower) = self.lower_bound else {
            return 0.0;
        };
        let Some(upper) = self.upper_bound else {
            return 0.0;
        };

        let blocks_in_range = (upper as Coord) > y_low && (lower as Coord) < y_high;
        if !blocks_in_range {
            return 0.0;
        }

        // Note: the kernel is small, so only a handful of blocks will contribute anything
        // here - maybe consider some kind of pruning
        //
        // 21.5. Tried getting the starting element through binary search
        //     to save some iterations, but it made it slower. I guess iterating over
        //     ~200 elements just isn't that bad?
        self.data.iter().fold(0.0, |acc, layer| {
            let layer_base = layer.base_height as Real;
            let layer_low = (layer_base).max(y_low as Real);
            let layer_high = (layer_base + layer.height as Real).min(y_high as Real);

            let layer_in_range = layer_high > layer_low;
            if !layer_in_range {
                return acc;
            }

            let layer_size = layer_high - layer_low;
            acc + layer_size
        })
    }

    // The block layers are ordered from low y to high y
    // so lower bound is the base of the first layer
    // and upper bound is the top of the last layer
    fn update_bounds(&mut self) {
        let init_lower_bound = self.lower_bound.is_none() && self.data.len() == 1;
        if init_lower_bound {
            self.lower_bound = Some(self.data[0].base_height);
        }

        let last_element = self
            .data
            .last()
            .expect("There should be a layer present in the tower!");
        self.upper_bound = Some(last_element.base_height + last_element.height as isize);
    }

    pub fn push(&mut self, block: BlockType, base_height: isize) {
        // We do not want to store Air blocks for now
        debug_assert!(block != BlockType::Air);

        let extend_top_layer = match self.data.last() {
            Some(layer) => {
                // Extend the layer if materials match and there is no air gap between the layers
                layer.material == block
                    && (layer.base_height + layer.height as isize) == base_height
            }
            None => false,
        };

        if extend_top_layer {
            // Should always be Some(..) if the check above passed
            if let Some(top_layer) = self.data.last_mut() {
                top_layer.height += 1;
            } else {
                println!("Something weird is going on..");
            }
            return;
        }

        let segment = MaterialLayer {
            material: block,
            height: 1,
            base_height,
        };

        self.data.push(segment);
        self.update_bounds();
    }
}
