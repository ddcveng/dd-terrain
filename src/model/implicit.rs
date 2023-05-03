use cgmath::Point2;
use cgmath::Point3;

use crate::discrete::World;

use super::rectangle::Rectangle;

// Radius of the cube used as the convolution kernel
const SIGMA: f32 = 1.0;
const KERNEL_VOLUME: f32 = 8.0 * SIGMA * SIGMA * SIGMA;
const KERNEL_VOLUME_HALF: f32 = KERNEL_VOLUME / 2.0;

pub const KERNEL_SIZE: f32 = 2.0 * SIGMA;

#[derive(Copy, Clone)]
pub struct Kernel {
    pub position: Point3<f32>,
}

impl Kernel {
    pub fn get_topdown_bottom_left_position(&self) -> Point2<f32> {
        let mut pos = self.position.xz();
        pos.x -= SIGMA;
        pos.y -= SIGMA;

        pos
    }

    pub fn get_bounding_rectangle(&self) -> Rectangle {
        let origin = Point2 {
            x: self.position.x - SIGMA,
            y: self.position.z - SIGMA,
        };

        Rectangle::square(origin, KERNEL_SIZE)
    }

    pub fn y_low(&self) -> f32 {
        self.position.y - SIGMA
    }

    pub fn y_high(&self) -> f32 {
        self.position.y + SIGMA
    }
}

// 2 * (material_volume / kernel_volume) - 1
// returns values in range [-1., 1.]
pub fn evaluate_density(model: &World, kernel: Kernel) -> f32 {
    return model.sample_volume(kernel) / KERNEL_VOLUME_HALF - 1.0;
}
