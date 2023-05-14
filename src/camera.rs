use crate::{
    config,
    input::Direction,
    model::{Position, Real},
    InputAction, InputConsumer, RenderState,
};
use cgmath::{
    perspective, Angle, InnerSpace, Matrix4, Point3, Rad, SquareMatrix, Vector2, Vector3,
};

pub struct Camera {
    pub world_to_view: Matrix4<Real>,
    pub view_to_world: Matrix4<Real>,
    pub projection: Matrix4<Real>,
    translation: Vector3<Real>,
    rotation: Option<Vector2<Real>>,
    fovy: Rad<Real>,
    aspect_ratio: Real,
    near_clipping_plane: Real,
    far_clipping_plane: Real,
}

impl Camera {
    pub fn new(
        position: Position,
        look_at: Position,
        world_up_vector: Vector3<Real>,
        fovy: Rad<Real>,
        aspect_ratio: Real,
        near_clipping_plane: Real,
        far_clipping_plane: Real,
    ) -> Self {
        let projection = perspective(fovy, aspect_ratio, near_clipping_plane, far_clipping_plane);
        let view = Matrix4::<Real>::look_at_rh(position, look_at, world_up_vector);
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

    pub fn update(&mut self, delta_time: Real) {
        let direction = self.view_to_world.z;

        let mut yaw: Rad<Real> = Angle::atan2(direction.z, direction.x);
        let mut pitch: Rad<Real> = Angle::asin(direction.y);

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
        aside: Vector3<Real>,
        up: Vector3<Real>,
        dir: Vector3<Real>,
        delta_time: Real,
    ) -> Position {
        let position = Point3::from_homogeneous(self.view_to_world.w);
        let mut new_position = position;
        new_position += aside * self.translation.x * delta_time;
        new_position += up * self.translation.y * delta_time;
        new_position += dir * self.translation.z * delta_time;

        new_position
    }

    pub fn get_position(&self) -> Position {
        return Point3::from_homogeneous(self.view_to_world.w);
    }

    // TODO: is this '-' here ok, or is my matrix wrong?
    pub fn get_direction(&self) -> Vector3<Real> {
        return -self.view_to_world.z.truncate();
    }

    fn update_aspect(&mut self, aspect_ratio: Real) {
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
                self.update_aspect(*width as Real / *height as Real)
            }
            _ => (),
        }

        if !state.cursor_captured {
            return;
        }

        if let InputAction::CursorMoved { x, y } = action {
            let rotation_direction =
                Vector2::new(*x as Real, *y as Real).normalize_to(config::SPHERE_RADIUS);

            self.rotation = Some(rotation_direction);
        }
    }
}
