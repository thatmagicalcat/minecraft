use crate::defer;
use glow::{HasContext, NativeBuffer};

const F32S: usize = std::mem::size_of::<f32>();

pub struct Cubes<'a> {
    gl: &'a glow::Context,
    vao: glow::NativeVertexArray,
    vbo: glow::NativeBuffer,
    instance_vbo: glow::NativeBuffer,
    instances: usize,
}

impl<'a> Cubes<'a> {
    pub fn new(gl: &'a glow::Context, instance_positions: &[f32]) -> Self {
        let cube = Self::init(gl, instance_positions.len());

        {
            cube.bind_vao();
            defer! { cube.unbind_vao(); }

            cube.bind_vbo(cube.vbo);

            cube.fill_buffer();
            cube.setup_attrib_ptrs();

            cube.bind_vbo(cube.instance_vbo);
            cube.setup_instance_vbo(instance_positions);

            // we only need one
            defer! { cube.unbind_vbo(); }
        }

        cube
    }

    pub fn render(&self) {
        unsafe {
            self.bind_vao();
            self.gl
                .draw_arrays_instanced(glow::TRIANGLES, 0, 36, self.instances as _);
        }
    }

    fn init(gl: &'a glow::Context, instances: usize) -> Self {
        let vao = unsafe { gl.create_vertex_array().unwrap() };
        let vbo = unsafe { gl.create_buffer().unwrap() };
        let instance_vbo = unsafe { gl.create_buffer().unwrap() };

        Self {
            gl,
            vao,
            vbo,
            instance_vbo,
            instances,
        }
    }

    fn bind_vbo(&self, vbo: NativeBuffer) {
        unsafe { self.gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo)) };
    }

    fn bind_vao(&self) {
        unsafe { self.gl.bind_vertex_array(Some(self.vao)) };
    }

    fn unbind_vao(&self) {
        unsafe { self.gl.bind_vertex_array(None) };
    }

    fn unbind_vbo(&self) {
        unsafe { self.gl.bind_buffer(glow::ARRAY_BUFFER, None) };
    }

    /// Upload vertex data.
    /// VAO must be bound before calling this.
    fn fill_buffer(&self) {
        assert!(self.is_vao_bound(), "VAO not bound");

        unsafe {
            self.gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
            self.gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(&VERTICES),
                glow::STATIC_DRAW,
            );
        }
    }

    /// Setup attribute points.
    /// VBO must be bound before calling this.
    fn setup_attrib_ptrs(&self) {
        assert!(self.is_vbo_bound(self.vbo), "VBO not bound");

        unsafe {
            // position
            self.gl.enable_vertex_attrib_array(0);
            self.gl
                .vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, STRIDE as i32, 0);

            // tex coord
            self.gl.enable_vertex_attrib_array(1);
            self.gl.vertex_attrib_pointer_f32(
                1,
                2,
                glow::FLOAT,
                false,
                STRIDE as i32,
                3 * F32S as i32,
            );

            // normal
            self.gl.enable_vertex_attrib_array(2);
            self.gl.vertex_attrib_pointer_f32(
                2,
                3,
                glow::FLOAT,
                false,
                STRIDE as i32,
                5 * F32S as i32,
            );
        }
    }

    /// Setup instance buffer.
    /// Instance VBO must be bound before calling this.
    fn setup_instance_vbo(&self, instance_positions: &[f32]) {
        assert!(
            self.is_vbo_bound(self.instance_vbo),
            "Instance VBO not bound"
        );

        unsafe {
            self.gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(instance_positions),
                glow::STATIC_DRAW,
            );

            // instance position
            self.gl.enable_vertex_attrib_array(3);
            self.gl
                .vertex_attrib_pointer_f32(3, 3, glow::FLOAT, false, F32S as i32 * 3, 0);
            self.gl.vertex_attrib_divisor(3, 1);
        }
    }

    fn is_vao_bound(&self) -> bool {
        let glow::NativeVertexArray(vao) = self.vao;
        unsafe { self.gl.get_parameter_i32(glow::VERTEX_ARRAY_BINDING) == vao.get() as i32 }
    }

    fn is_vbo_bound(&self, vbo: NativeBuffer) -> bool {
        let glow::NativeBuffer(vbo) = vbo;
        unsafe { self.gl.get_parameter_i32(glow::ARRAY_BUFFER_BINDING) == vbo.get() as i32 }
    }
}

impl Drop for Cubes<'_> {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_vertex_array(self.vao);
            self.gl.delete_buffer(self.vbo);
            self.gl.delete_buffer(self.instance_vbo);
        }
    }
}

const STRIDE: usize = 8 * F32S;

// x, y, z, s, t, nx, ny, nz
#[rustfmt::skip]
const VERTICES: [f32; 288] = [
    // ---------------------------
    // FRONT face (z = +0.5), normal (0, 0, 1)
    // Triangle 1
    -0.5, -0.5,  0.5,  0.50, 0.3333,  0.0,  0.0,  1.0, // bottom-left
    -0.5,  0.5,  0.5,  0.50, 0.6667,  0.0,  0.0,  1.0, // top-left
     0.5,  0.5,  0.5,  0.75, 0.6667,  0.0,  0.0,  1.0, // top-right
    // Triangle 2
    -0.5, -0.5,  0.5,  0.50, 0.3333,  0.0,  0.0,  1.0, // bottom-left
     0.5,  0.5,  0.5,  0.75, 0.6667,  0.0,  0.0,  1.0, // top-right
     0.5, -0.5,  0.5,  0.75, 0.3333,  0.0,  0.0,  1.0, // bottom-right

    // ---------------------------
    // BACK face (z = -0.5), normal (0, 0, -1)
    // Triangle 1
     0.5, -0.5, -0.5,  0.00, 0.3333,  0.0,  0.0, -1.0,
     0.5,  0.5, -0.5,  0.00, 0.6667,  0.0,  0.0, -1.0,
    -0.5,  0.5, -0.5,  0.25, 0.6667,  0.0,  0.0, -1.0,
    // Triangle 2
     0.5, -0.5, -0.5,  0.00, 0.3333,  0.0,  0.0, -1.0,
    -0.5,  0.5, -0.5,  0.25, 0.6667,  0.0,  0.0, -1.0,
    -0.5, -0.5, -0.5,  0.25, 0.3333,  0.0,  0.0, -1.0,

    // ---------------------------
    // LEFT face (x = -0.5), normal (-1, 0, 0)
    // Triangle 1
    -0.5, -0.5, -0.5,  0.25, 0.3333, -1.0,  0.0,  0.0,
    -0.5,  0.5, -0.5,  0.25, 0.6667, -1.0,  0.0,  0.0,
    -0.5,  0.5,  0.5,  0.50, 0.6667, -1.0,  0.0,  0.0,
    // Triangle 2
    -0.5, -0.5, -0.5,  0.25, 0.3333, -1.0,  0.0,  0.0,
    -0.5,  0.5,  0.5,  0.50, 0.6667, -1.0,  0.0,  0.0,
    -0.5, -0.5,  0.5,  0.50, 0.3333, -1.0,  0.0,  0.0,

    // ---------------------------
    // RIGHT face (x =  0.5), normal (1, 0, 0)
    // Triangle 1
     0.5, -0.5,  0.5,  0.75, 0.3333,  1.0,  0.0,  0.0,
     0.5,  0.5,  0.5,  0.75, 0.6667,  1.0,  0.0,  0.0,
     0.5,  0.5, -0.5,  1.00, 0.6667,  1.0,  0.0,  0.0,
    // Triangle 2
     0.5, -0.5,  0.5,  0.75, 0.3333,  1.0,  0.0,  0.0,
     0.5,  0.5, -0.5,  1.00, 0.6667,  1.0,  0.0,  0.0,
     0.5, -0.5, -0.5,  1.00, 0.3333,  1.0,  0.0,  0.0,

    // ---------------------------
    // UP face (y =  0.5), normal (0, 1, 0)
    // Triangle 1
    -0.5,  0.5,  0.5,  0.25, 0.6667,  0.0,  1.0,  0.0,
    -0.5,  0.5, -0.5,  0.25, 1.00,    0.0,  1.0,  0.0,
     0.5,  0.5, -0.5,  0.50, 1.00,    0.0,  1.0,  0.0,
    // Triangle 2
    -0.5,  0.5,  0.5,  0.25, 0.6667,  0.0,  1.0,  0.0,
     0.5,  0.5, -0.5,  0.50, 1.00,    0.0,  1.0,  0.0,
     0.5,  0.5,  0.5,  0.50, 0.6667,  0.0,  1.0,  0.0,

    // ---------------------------
    // DOWN face (y = -0.5), normal (0, -1, 0)
    // Triangle 1
    -0.5, -0.5, -0.5,  0.25, 0.00,  0.0, -1.0,  0.0,
    -0.5, -0.5,  0.5,  0.25, 0.3333,  0.0, -1.0,  0.0,
     0.5, -0.5,  0.5,  0.50, 0.3333,  0.0, -1.0,  0.0,
    // Triangle 2
    -0.5, -0.5, -0.5,  0.25, 0.00,  0.0, -1.0,  0.0,
     0.5, -0.5,  0.5,  0.50, 0.3333,  0.0, -1.0,  0.0,
     0.5, -0.5, -0.5,  0.50, 0.00,  0.0, -1.0,  0.0,
];
