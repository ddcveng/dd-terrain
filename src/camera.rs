use crate::{input::Direction, InputAction, InputConsumer, RenderState};
use cgmath::{
    perspective, Angle, InnerSpace, Matrix4, Point2, Point3, Rad, SquareMatrix, Vector2, Vector3,
    Vector4, Zero,
};

const CAMERA_MOVE_SPEED: f32 = 2.0;
const ROTATION_SPEED: Rad<f32> = Rad(std::f32::consts::PI);
const SPHERE_RADIUS: f32 = 5.0;
const EPSILON: f32 = 0.1;

pub struct Camera {
    pub world_to_view: Matrix4<f32>,
    pub view_to_world: Matrix4<f32>,
    pub projection: Matrix4<f32>,
    translation: Vector3<f32>,
    rotation: Option<Vector2<f32>>,
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
        let view_inverse = view.invert().unwrap();

        Camera {
            world_to_view: view,
            view_to_world: view_inverse,
            projection,
            translation: Vector3::new(0., 0., 0.),
            rotation: None,
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        let translation = Matrix4::from_translation(self.translation * delta_time);
        self.world_to_view = translation * self.world_to_view;

        // TODO: use quaternions for nicer rotations
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
            self.world_to_view = rotation * self.world_to_view;
            self.rotation = None;
        }

        self.view_to_world = self.world_to_view.invert().unwrap();
    }

    pub fn get_position(&self) -> Point3<f32> {
        return Point3::from_homogeneous(self.view_to_world.w);
    }

    pub fn get_direction(&self) -> Vector3<f32> {
        return self.view_to_world.z.truncate();
    }
}

impl InputConsumer for Camera {
    fn consume(&mut self, action: &InputAction, state: &RenderState) -> () {
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

        if !state.cursor_captured {
            return;
        }

        if let InputAction::CursorMoved { x, y } = action {
            let rotation_direction = Vector2::new(*x as f32, *y as f32)
                .normalize_to(SPHERE_RADIUS);

            self.rotation = Some(rotation_direction);
        }
    }
}
