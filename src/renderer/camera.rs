#[derive(Debug)]
pub struct Camera {
    vfov: f32,
    near_plane: f32,
    far_plane: f32,

    projection: glam::Mat4,
    view: glam::Mat4,

    position: glam::Vec3,
    forward_direction: glam::Vec3,

    viewport_height: u32,
    viewport_width: u32,

    last_mouse_position: glam::Vec2,
}

impl Camera {
    pub fn new(
        position: glam::Vec3,
        vfov: f32,
        near_plane: f32,
        far_plane: f32,
        viewport_width: u32,
        viewport_height: u32,
    ) -> Self {
        Self {
            vfov,
            near_plane,
            far_plane,
            position,

            viewport_height,
            viewport_width,

            projection: glam::Mat4::IDENTITY,
            view: glam::Mat4::IDENTITY,

            forward_direction: glam::vec3(0.0, 0.0, -1.0),

            last_mouse_position: glam::Vec2::ZERO,
        }
    }

    pub fn update(&mut self, dt: f32, pointer_state: PointerState, keyboard_state: KeyboardState) {
        let Some(mouse_pos) = pointer_state.pos else {
            return;
        };

        if !pointer_state.secondary_down {
            self.last_mouse_position = mouse_pos;
            return;
        }

        let mouse_delta = (mouse_pos - self.last_mouse_position) * 0.002;
        self.last_mouse_position = mouse_pos;

        let up_direction = glam::Vec3::Y;
        let speed = 5.0;
        let right_direction = self.forward_direction.cross(up_direction);

        let mut moved = false;
        let mut f = |c: bool, v: glam::Vec3| {
            if c {
                self.position += v * speed * dt;
                moved = true;
            }
        };

        // translation
        f(keyboard_state.w, self.forward_direction);
        f(keyboard_state.s, -self.forward_direction);
        f(keyboard_state.a, -right_direction);
        f(keyboard_state.d, right_direction);
        f(keyboard_state.q, -up_direction);
        f(keyboard_state.e, up_direction);

        // rotation
        if mouse_delta.x != 0.0 || mouse_delta.y != 0.0 {
            let pitch_delta = mouse_delta.y * self.get_rotation_speed();
            let yaw_delta = mouse_delta.x * self.get_rotation_speed();

            let q = (glam::Quat::from_axis_angle(right_direction, -pitch_delta)
                * glam::Quat::from_axis_angle(glam::Vec3::Y, -yaw_delta))
            .normalize();

            self.forward_direction = q * self.forward_direction;

            moved = true;
        }

        if moved {
            self.recalculate_view();
        }
    }

    pub const fn get_rotation_speed(&self) -> f32 {
        0.3
    }

    pub fn resize(&mut self, new_width: u32, new_height: u32) {
        if self.viewport_height == new_height && self.viewport_width == new_width {
            return;
        }

        self.viewport_height = new_height;
        self.viewport_width = new_width;

        self.recalculate_projection();
    }

    pub fn recalculate_view(&mut self) {
        self.view = glam::Mat4::look_at_rh(
            self.position,
            self.position + self.forward_direction,
            glam::Vec3::Y,
        );
    }

    pub fn recalculate_projection(&mut self) {
        self.projection = glam::Mat4::perspective_rh(
            self.vfov,
            self.viewport_width as f32 / self.viewport_height as f32,
            self.near_plane,
            self.far_plane,
        );
    }

    pub fn get_projection(&self) -> &glam::Mat4 {
        &self.projection
    }

    pub fn get_view(&self) -> &glam::Mat4 {
        &self.view
    }

    pub fn get_position(&self) -> &glam::Vec3 {
        &self.position
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PointerState {
    pub pos: Option<glam::Vec2>,
    pub secondary_down: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct KeyboardState {
    pub w: bool,
    pub a: bool,
    pub s: bool,
    pub d: bool,
    pub q: bool,
    pub e: bool,
}

// #[allow(unused)]
// pub struct Camera {
//     pub eye: glam::Vec3,
//     pub target: glam::Vec3,
//     pub up: glam::Vec3,
//     pub aspect: f32,
//     pub fovy: f32,
//     pub z_near: f32,
//     pub z_far: f32,
//     pub speed: f32,

//     pub forward_pressed: bool,
//     pub backward_pressed: bool,
//     pub left_pressed: bool,
//     pub right_pressed: bool,
// }

// impl Camera {
//     pub fn get_view(&self) -> glam::Mat4 {
//         glam::Mat4::look_at_rh(self.eye, self.target, self.up)
//     }

//     pub fn get_projection(&self) -> glam::Mat4 {
//         glam::Mat4::perspective_rh_gl(self.fovy, self.aspect, self.z_near, self.z_far)
//     }

//     pub fn process_event(&mut self, event: &glfw::WindowEvent) -> bool {
//         match event {
//             glfw::WindowEvent::Key(key, _, action, _) => {
//                 let value = matches!(action, glfw::Action::Press | glfw::Action::Repeat);

//                 match key {
//                     glfw::Key::W => self.forward_pressed = value,
//                     glfw::Key::A => self.left_pressed = value,
//                     glfw::Key::S => self.backward_pressed = value,
//                     glfw::Key::D => self.right_pressed = value,

//                     _ => return false,
//                 }
//             }

//             _ => return false,
//         };

//         let forward = self.target - self.eye;
//         let forward_norm = forward.normalize();

//         if self.forward_pressed {
//             self.eye += forward_norm * self.speed;
//         }

//         if self.backward_pressed {
//             self.eye -= forward_norm * self.speed;
//         }

//         let right = forward_norm.cross(self.up);

//         let forward = self.target - self.eye;
//         let forward_mag = forward.length();

//         if self.right_pressed {
//             self.eye = self.target - (forward - right * self.speed).normalize() * forward_mag;
//         }

//         if self.left_pressed {
//             self.eye = self.target - (forward + right * self.speed).normalize() * forward_mag;
//         }

//         true
//     }
// }
