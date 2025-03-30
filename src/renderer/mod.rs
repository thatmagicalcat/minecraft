use glow::HasContext;

mod camera;
mod cube;
mod light;
mod program_manager;
mod texture;

use cube::Cubes;
use program_manager::Program;
use texture::TextureData;

pub use camera::*;
pub use light::Light;

const TEXTURE_WIDTH: usize = 64;
const TEXTURE_HEIGHT: usize = 48;

pub struct Renderer<'a> {
    gl: &'a glow::Context,
    cubes: Cubes<'a>,

    // texture_id: glow::NativeTexture,
    texture_array_id: glow::NativeTexture,

    camera: Camera,
    light: Light<'a>,
    program: Program<'a>,
}

impl<'a> Renderer<'a> {
    pub fn new(
        gl: &'a glow::Context,
        camera: Camera,
        instance_positions: &[f32],
        instance_texture_ids: &[i32],
        light_color: glam::Vec3,
        light_position: glam::Vec3,
    ) -> Self {
        // let texture_data = TextureData::new("res/dirt.png");
        // let texture_id = unsafe {
        //     let id = gl.create_texture().unwrap();
        //     gl.bind_texture(glow::TEXTURE_2D, Some(id));

        // gl.tex_image_2d(
        //     glow::TEXTURE_2D,
        //     0,
        //     texture_data.format as _,
        //     texture_data.width,
        //     texture_data.height,
        //     0,
        //     texture_data.format,
        //     glow::UNSIGNED_BYTE,
        //     glow::PixelUnpackData::Slice(Some(texture_data.data)),
        // );

        //     gl.generate_mipmap(glow::TEXTURE_2D);

        //     texture::setup_texture_params(gl, glow::TEXTURE_2D);

        //     id
        // };

        let program = Program::from_str(
            gl,
            include_str!("../shader/basic.glsl"),
            "vertex",
            "fragment",
        )
        .expect("failed to create shader program");

        let texture_array_id = unsafe { gl.create_texture().unwrap() };
        let texture_names = ["res/grass.png", "res/dirt.png"];

        unsafe {
            gl.bind_texture(glow::TEXTURE_2D_ARRAY, Some(texture_array_id));
            gl.tex_storage_3d(
                glow::TEXTURE_2D_ARRAY,
                1,
                glow::RGB8,
                TEXTURE_WIDTH as _,
                TEXTURE_HEIGHT as _,
                texture_names.len() as _,
            );

            for (z_offset, name) in texture_names.iter().enumerate() {
                let data = TextureData::new(name);
                gl.tex_sub_image_3d(
                    glow::TEXTURE_2D_ARRAY,
                    0,
                    0,
                    0,
                    z_offset as _,
                    TEXTURE_WIDTH as _,
                    TEXTURE_HEIGHT as _,
                    1,
                    data.format,
                    glow::UNSIGNED_BYTE,
                    glow::PixelUnpackData::Slice(Some(data.data)),
                );
            }

            gl.generate_mipmap(glow::TEXTURE_2D_ARRAY);
            texture::setup_texture_params(gl, glow::TEXTURE_2D_ARRAY);
        }

        Self {
            gl,
            texture_array_id,
            camera,
            program,

            light: Light::new(gl, light_position, light_color),
            cubes: Cubes::new(gl, instance_positions, instance_texture_ids),
        }
    }

    pub fn render(&mut self) {
        self.program.use_program();
        self.set_uniforms();
        self.bind_texture();
        self.cubes.render();
    }

    pub fn resize_camera(&mut self, new_width: u32, new_height: u32) {
        self.camera.resize(new_width, new_height);
        unsafe { self.gl.viewport(0, 0, new_width as _, new_height as _) };
    }

    pub fn update(&mut self, dt: f32, pointer_state: PointerState, keyboard_state: KeyboardState) {
        self.camera.update(dt, pointer_state, keyboard_state);
    }

    fn bind_texture(&self) {
        unsafe {
            self.gl
                .bind_texture(glow::TEXTURE_2D_ARRAY, Some(self.texture_array_id));
        }
    }

    fn set_uniforms(&mut self) {
        self.camera.recalculate_view();
        self.camera.recalculate_projection();

        let view = self.camera.get_view();
        let projection = self.camera.get_projection();

        self.light.set_uniforms(&self.program);

        unsafe {
            // eye position
            let &glam::Vec3 { x, y, z } = self.camera.get_position();
            self.gl.uniform_3_f32(
                self.program.get_uniform_location("eye_position").as_ref(),
                x,
                y,
                z,
            );

            // view matrix
            self.gl.uniform_matrix_4_f32_slice(
                self.program.get_uniform_location("view").as_ref(),
                false,
                &view.to_cols_array(),
            );

            // projection matrix
            self.gl.uniform_matrix_4_f32_slice(
                self.program.get_uniform_location("projection").as_ref(),
                false,
                &projection.to_cols_array(),
            );
        }
    }
}

impl Drop for Renderer<'_> {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_texture(self.texture_array_id);
        }
    }
}
