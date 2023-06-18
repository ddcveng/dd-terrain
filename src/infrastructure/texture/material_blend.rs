use crate::model::{
    common::{activation_treshold, BlockType, BLOCK_TYPES},
    Real,
};
use array_init::array_init;

type MaterialWeights = [Real; BLOCK_TYPES];
pub struct MaterialBlend {
    material_contributions: MaterialWeights,
    contributed: Real,
}

impl MaterialBlend {
    pub fn new() -> Self {
        MaterialBlend {
            material_contributions: [0.0; BLOCK_TYPES],
            contributed: 0.0,
        }
    }

    pub fn mix(&mut self, material: BlockType, amount: Real) {
        let material_index = material as usize;

        self.material_contributions[material_index] += amount;
        self.contributed += amount;
    }

    pub fn merge(&mut self, other: MaterialBlend) {
        for (base, other_val) in self
            .material_contributions
            .iter_mut()
            .zip(other.material_contributions)
        {
            *base += other_val;
        }

        self.contributed += other.contributed;
    }

    pub fn into_material_weights(self) -> [[f32; 4]; 4] {
        let mut weights_flat = self.normalized_weights();

        let redistribute = Self::has_active_materials(&weights_flat);
        if redistribute {
            Self::redistribute_inactive_weights(&mut weights_flat);
        }

        let weights = array_init(|col| {
            array_init(|row| {
                let contribution_index = col * 4 + row;

                // There are not 16 block types yet ...
                if contribution_index < weights_flat.len() {
                    weights_flat[contribution_index] as f32
                } else {
                    0.0
                }
            })
        });

        return weights;
    }

    fn normalized_weights(self) -> MaterialWeights {
        array_init(|i| {
            let weight = self.material_contributions[i];
            let normalized_weight = weight / self.contributed;

            normalized_weight
        })
    }

    fn has_active_materials(weights: &[Real; BLOCK_TYPES]) -> bool {
        let any_active_materials = weights.iter().enumerate().any(|(i, w)| {
            if let Ok(block_type) = i.try_into() {
                let treshold = activation_treshold(block_type);
                *w > treshold
            } else {
                false
            }
        });

        any_active_materials
    }

    fn redistribute_inactive_weights(weights_flat: &mut MaterialWeights) {
        let mut redistributed_weight = 0.0;
        let mut dominant_material_index = 0;

        for material_index in 0..BLOCK_TYPES {
            let Ok(block_type) = material_index.try_into() else {
                continue;
            };

            let w = weights_flat[material_index];
            let treshold = activation_treshold(block_type);

            let material_active = w > treshold;
            if material_active {
                let max_w = weights_flat[dominant_material_index];
                if w > max_w {
                    dominant_material_index = material_index;
                }
            } else {
                weights_flat[material_index] = 0.0;
                redistributed_weight += w;
            }
        }

        weights_flat[dominant_material_index] += redistributed_weight;
    }
}
