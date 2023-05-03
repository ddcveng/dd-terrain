use cgmath::Point2;

#[derive(Copy, Clone)]
pub struct Rectangle {
    pub origin: Point2<f32>,
    pub width: f32,
    pub height: f32,
}

const EPSILON: f32 = 0.0001;

impl Rectangle {
    pub fn square(origin: Point2<f32>, size: f32) -> Self {
        Rectangle {
            origin,
            width: size,
            height: size,
        }
    }

    pub fn intersect(&self, other: Self) -> Option<Self> {
        let intersection_origin = Point2 {
            x: self.left().max(other.left()),
            y: self.bottom().max(other.bottom()),
        };

        let intersection_right = self.right().min(other.right());
        let intersection_top = self.top().min(other.top());
        let intersection_width = intersection_right - intersection_origin.x;
        let intersection_height = intersection_top - intersection_origin.y;

        let no_intersection = intersection_width < EPSILON || intersection_height < EPSILON;
        if no_intersection {
            return None;
        }

        Some(Rectangle {
            origin: intersection_origin,
            width: intersection_width,
            height: intersection_height,
        })
    }

    pub fn offset_origin(self, offset: Point2<f32>) -> Self {
        let offset_origin = Point2 {
            x: self.origin.x + offset.x,
            y: self.origin.y + offset.y,
        };

        Rectangle {
            origin: offset_origin,
            width: self.width,
            height: self.height,
        }
    }

    pub fn left(&self) -> f32 {
        self.origin.x
    }
    pub fn right(&self) -> f32 {
        self.origin.x + self.width
    }
    pub fn top(&self) -> f32 {
        self.origin.y + self.height
    }
    pub fn bottom(&self) -> f32 {
        self.origin.y
    }
}
