use crate::model::{
    common::{BlockType, BLOCK_TYPES},
    Real,
};

const EPSILON: f32 = 0.0001;

pub struct MaterialBlend {
    material_contributions: [Real; BLOCK_TYPES],
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

    pub fn into_material_weights(self) -> ([f32; 4], [u8; 4]) {
        let mut weights = [0.0; 4];
        let mut indices: [u8; 4] = [0; 4];

        for (index, weight) in self.material_contributions.into_iter().enumerate() {
            let smallest_index = get_index_of_smallest_weight(&weights);
            if weight > weights[smallest_index] {
                weights[smallest_index] = weight;
                indices[smallest_index] = index as u8;
            }
        }

        let used_contributions: Real = weights.iter().sum();
        let missing_contributions = self.contributed - used_contributions;

        let contribution_correction = missing_contributions / 4.0;
        let weights = array_init::array_init(|i| {
            ((weights[i] + contribution_correction) / self.contributed) as f32
        });

        debug_assert!(weights
            .iter()
            .all(|w| (*w > -EPSILON) && (*w < 1.0 + EPSILON)));
        debug_assert!(
            (weights.iter().sum::<f32>() - 1.0).abs() < EPSILON,
            "Weights do not add up to 1 - {:?} {:?} {}",
            &weights,
            &indices,
            self.contributed
        );

        (weights, indices)
    }
}

fn get_index_of_smallest_weight(w: &[Real; 4]) -> usize {
    let mut index = 0;
    let mut smallest: Real = w[index];

    for i in 1..4 {
        if w[i] < smallest {
            index = i;
            smallest = w[index];
        }
    }

    index
}
