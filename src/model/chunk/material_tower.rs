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

        self.data.iter().fold(0.0, |acc, layer| {
            let layer_base = layer.base_height as Real;
            let layer_low = (layer_base).max(y_low as Real);
            let layer_high = (layer_base + layer.height as Real).min(y_high as Real);

            let layer_size = (layer_high - layer_low).max(0.0);
            acc + layer_size
        })
    }

    pub fn get_layers_in_range(
        &self,
        y_low: Coord,
        y_high: Coord,
    ) -> impl Iterator<Item = (BlockType, Real)> + '_ {
        let layers_in_range = self.data.iter().filter_map(move |layer| {
            let layer_low = (layer.base_height as Real).max(y_low as Real);
            let layer_high = (layer.base_height as Real + layer.height as Real).min(y_high as Real);
            let layer_in_range = layer_high > layer_low;
            if !layer_in_range {
                return None;
            }

            let new_height = layer_high - layer_low;
            Some((layer.material, new_height))
        });

        layers_in_range
        //return layers_in_range.collect();
    }

    pub fn push(&mut self, block: BlockType, base_height: isize) {
        // We do not want to store Air blocks for now
        debug_assert!(block != BlockType::Air);
        if self.data.is_empty() {
            self.lower_bound = Some(base_height);
        }

        self.upper_bound = {
            let current_bound = self.upper_bound.unwrap_or(base_height);
            Some(current_bound + 1)
        };

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
    }
}
