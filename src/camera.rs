use crate::{input::Direction, InputAction, InputConsumer};
use cgmath::{
    perspective, Angle, InnerSpace, Matrix4, Point2, Point3, Rad, SquareMatrix, Vector2, Vector3,
    Vector4, Zero,
};

const CAMERA_MOVE_SPEED: f32 = 2.0;
const ROTATION_SPEED: Rad<f32> = Rad(std::f32::consts::PI);
const SPHERE_RADIUS: f32 = 5.0;
const EPSILON: f32 = 0.1;

pub struct Camera {
    pub view: Matrix4<f32>,
    pub projection: Matrix4<f32>,
    translation: Vector3<f32>,
    rotation: Option<Vector2<f32>>,
    cursor_position: Point2<f64>,
    //    look_at_dir: Vector3<f32>,
    //    theta: Rad<f32>,
    //    phi: Rad<f32>,
}

impl Camera {
    pub fn new(
        position: Point3<f32>,
        look_at: Point3<f32>,
        world_up_vector: Vector3<f32>,
        fovy: Rad<f32>,
        aspect_ratio: f32,
        near_clipping_plane: f32,
        far_clipping_plane: f32,
    ) -> Self {
        let projection = perspective(fovy, aspect_ratio, near_clipping_plane, far_clipping_plane);
        let view = Matrix4::<f32>::look_at_rh(position, look_at, world_up_vector);

        Camera {
            view,
            projection,
            translation: Vector3::new(0., 0., 0.),
            rotation: None,
            cursor_position: Point2::new(0., 0.),
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        let translation = Matrix4::from_translation(self.translation * delta_time);
        self.view = translation * self.view;

        if let Some(rotation) = self.rotation {
            let rotation_parts_sum = rotation.x.abs() + rotation.y.abs();
            if rotation_parts_sum < EPSILON {
                return;
            }

            let pitch_factor = rotation.y.abs() / rotation_parts_sum;
            let pitch_sign = rotation.y.signum();
            let pitch = ROTATION_SPEED * pitch_factor * pitch_sign * delta_time;

            let yaw_sign = rotation.x.signum();
            let yaw = ROTATION_SPEED * (1.0 - pitch_factor) * yaw_sign * delta_time;

            let rotation = Matrix4::from_angle_x(pitch) * Matrix4::from_angle_y(yaw);
            self.view = rotation * self.view;
            self.rotation = None;
        }
    }
}

impl InputConsumer for Camera {
    fn consume(&mut self, action: &InputAction, delta_t: f32, cursor_captured: bool) -> () {
        match action {
            InputAction::BeginMove { dir } => match dir {
                Direction::Forward => self.translation.z = CAMERA_MOVE_SPEED,
                Direction::Back => self.translation.z = -CAMERA_MOVE_SPEED,
                Direction::Left => self.translation.x = CAMERA_MOVE_SPEED,
                Direction::Right => self.translation.x = -CAMERA_MOVE_SPEED,
                Direction::Up => self.translation.y = -CAMERA_MOVE_SPEED,
                Direction::Down => self.translation.y = CAMERA_MOVE_SPEED,
            },
            InputAction::EndMove { dir } => match dir {
                Direction::Forward => self.translation.z = 0.0,
                Direction::Back => self.translation.z = 0.0,
                Direction::Left => self.translation.x = 0.0,
                Direction::Right => self.translation.x = 0.0,
                Direction::Up => self.translation.y = 0.0,
                Direction::Down => self.translation.y = 0.0,
            },
            _ => (),
        }

        if !cursor_captured {
            return;
        }

        if let InputAction::CursorMoved { x, y } = action {
            let new_cursor_position = Point2::new(*x, *y);
            let cursor_delta = new_cursor_position - self.cursor_position;
            let rotation_direction = Vector2::new(cursor_delta.x as f32, cursor_delta.y as f32)
                .normalize_to(SPHERE_RADIUS);

            self.rotation = Some(rotation_direction);
            self.cursor_position = new_cursor_position;
        }
    }
}
