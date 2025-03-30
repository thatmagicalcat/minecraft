use glow::HasContext;

pub struct Light<'a> {
    gl: &'a glow::Context,
    light_position: glam::Vec3,
    light_color: glam::Vec3,
    // no drawing for now
    // vao: glow::NativeVertexArray,
    // vbo: glow::NativeBuffer,
}

impl<'a> Light<'a> {
    pub fn new(gl: &'a glow::Context, light_position: glam::Vec3, light_color: glam::Vec3) -> Self {
        Self {
            gl,
            light_position,
            light_color,
        }
    }

    pub fn set_uniforms(&self, program: &super::Program) {
        unsafe {
            // light position
            let glam::Vec3 { x, y, z } = self.light_position;
            self.gl.uniform_3_f32(
                program.get_uniform_location("light_position").as_ref(),
                x,
                y,
                z,
            );

            // light color
            let glam::Vec3 { x: r, y: g, z: b } = self.light_color;
            self.gl.uniform_3_f32(
                program.get_uniform_location("light_color").as_ref(),
                r,
                g,
                b,
            );
        }
    }
}
