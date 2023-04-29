use crate::{config, input::Direction, InputAction, InputConsumer, RenderState};
use cgmath::{
    perspective, Angle, InnerSpace, Matrix4, Point3, Rad, SquareMatrix, Vector2, Vector3,
};

pub struct Camera {
    pub world_to_view: Matrix4<f32>,
    pub view_to_world: Matrix4<f32>,
    pub projection: Matrix4<f32>,
    translation: Vector3<f32>,
    rotation: Option<Vector2<f32>>,
    fovy: Rad<f32>,
    aspect_ratio: f32,
    near_clipping_plane: f32,
    far_clipping_plane: f32,
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
            fovy,
            aspect_ratio,
            near_clipping_plane,
            far_clipping_plane,
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        let direction = self.view_to_world.z;

        let mut yaw: Rad<f32> = Angle::atan2(direction.z, direction.x);
        let mut pitch: Rad<f32> = Angle::asin(direction.y);

        if let Some(rotation) = self.rotation {
            yaw += Rad(rotation.x * config::SENSITIVITY);
            pitch += Rad(rotation.y * config::SENSITIVITY);

            // TODO: avoid singularities

            self.rotation = None;
        }

        let new_direction = Vector3::new(
            pitch.cos() * yaw.cos(),
            pitch.sin(),
            pitch.cos() * yaw.sin(),
        )
        .normalize();

        let aside_3d = self.view_to_world.x.truncate();
        let up_3d = self.view_to_world.y.truncate();
        let new_position = self.new_position(aside_3d, up_3d, new_direction, delta_time);

        let world_up = Vector3::unit_y();
        let aside_3d = self.view_to_world.x.truncate();

        let view_to_world = Matrix4::from_cols(
            world_up.cross(new_direction).normalize().extend(0.0),
            new_direction.cross(aside_3d).normalize().extend(0.0),
            new_direction.extend(0.0),
            new_position.to_homogeneous(),
        );

        self.view_to_world = view_to_world;
        self.world_to_view = view_to_world.invert().unwrap();
    }

    fn new_position(
        &self,
        aside: Vector3<f32>,
        up: Vector3<f32>,
        dir: Vector3<f32>,
        delta_time: f32,
    ) -> Point3<f32> {
        let position = Point3::from_homogeneous(self.view_to_world.w);
        let mut new_position = position;
        new_position += aside * self.translation.x * delta_time;
        new_position += up * self.translation.y * delta_time;
        new_position += dir * self.translation.z * delta_time;

        new_position
    }

    pub fn get_position(&self) -> Point3<f32> {
        return Point3::from_homogeneous(self.view_to_world.w);
    }

    // TODO: is this '-' here ok, or is my matrix wrong?
    pub fn get_direction(&self) -> Vector3<f32> {
        return -self.view_to_world.z.truncate();
    }

    fn update_aspect(&mut self, aspect_ratio: f32) {
        self.aspect_ratio = aspect_ratio;

        self.projection = perspective(
            self.fovy,
            self.aspect_ratio,
            self.near_clipping_plane,
            self.far_clipping_plane,
        );
    }
}

impl InputConsumer for Camera {
    fn consume(&mut self, action: &InputAction, state: &RenderState) -> () {
        match action {
            InputAction::BeginMove { dir } => match dir {
                Direction::Forward => self.translation.z = -config::CAMERA_MOVE_SPEED,
                Direction::Back => self.translation.z = config::CAMERA_MOVE_SPEED,
                Direction::Left => self.translation.x = -config::CAMERA_MOVE_SPEED,
                Direction::Right => self.translation.x = config::CAMERA_MOVE_SPEED,
                Direction::Up => self.translation.y = config::CAMERA_MOVE_SPEED,
                Direction::Down => self.translation.y = -config::CAMERA_MOVE_SPEED,
            },
            InputAction::EndMove { dir } => match dir {
                Direction::Forward => self.translation.z = 0.0,
                Direction::Back => self.translation.z = 0.0,
                Direction::Left => self.translation.x = 0.0,
                Direction::Right => self.translation.x = 0.0,
                Direction::Up => self.translation.y = 0.0,
                Direction::Down => self.translation.y = 0.0,
            },
            InputAction::Resized(width, height) => {
                self.update_aspect(*width as f32 / *height as f32)
            }
            _ => (),
        }

        if !state.cursor_captured {
            return;
        }

        if let InputAction::CursorMoved { x, y } = action {
            let rotation_direction =
                Vector2::new(*x as f32, *y as f32).normalize_to(config::SPHERE_RADIUS);

            self.rotation = Some(rotation_direction);
        }
    }
}
