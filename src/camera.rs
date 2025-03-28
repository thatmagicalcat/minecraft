#[allow(unused)]
pub struct Camera {
    pub eye: glam::Vec3,
    pub target: glam::Vec3,
    pub up: glam::Vec3,
    pub aspect: f32,
    pub fovy: f32,
    pub z_near: f32,
    pub z_far: f32,
    pub speed: f32,

    pub forward_pressed: bool,
    pub backward_pressed: bool,
    pub left_pressed: bool,
    pub right_pressed: bool,
}

impl Camera {
    pub fn get_view(&self) -> glam::Mat4 {
        glam::Mat4::look_at_rh(self.eye, self.target, self.up)
    }

    pub fn get_projection(&self) -> glam::Mat4 {
        glam::Mat4::perspective_rh_gl(self.fovy, self.aspect, self.z_near, self.z_far)
    }

    pub fn process_event(&mut self, event: &glfw::WindowEvent) -> bool {
        match event {
            glfw::WindowEvent::Key(key, _, action, _) => {
                let value = matches!(action, glfw::Action::Press | glfw::Action::Repeat);

                match key {
                    glfw::Key::W => self.forward_pressed = value,
                    glfw::Key::A => self.left_pressed = value,
                    glfw::Key::S => self.backward_pressed = value,
                    glfw::Key::D => self.right_pressed = value,

                    _ => return false,
                }
            }

            _ => return false,
        };

        let forward = self.target - self.eye;
        let forward_norm = forward.normalize();

        if self.forward_pressed {
            self.eye += forward_norm * self.speed;
        }

        if self.backward_pressed {
            self.eye -= forward_norm * self.speed;
        }

        let right = forward_norm.cross(self.up);

        let forward = self.target - self.eye;
        let forward_mag = forward.length();

        if self.right_pressed {
            self.eye = self.target - (forward - right * self.speed).normalize() * forward_mag;
        }

        if self.left_pressed {
            self.eye = self.target - (forward + right * self.speed).normalize() * forward_mag;
        }

        true
    }
}
