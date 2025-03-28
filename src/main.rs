use std::ffi::CString;

use camera::Camera;
use glfw::{WindowEvent, *};
use glow::*;

mod camera;

use stb_image::stb_image as image;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 800;

const F32S: usize = std::mem::size_of::<f32>();

fn main() {
    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();

    glfw.window_hint(WindowHint::ContextVersionMinor(3));
    glfw.window_hint(WindowHint::ContextVersionMajor(3));
    glfw.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));
    glfw.window_hint(WindowHint::Decorated(false)); // No frame
    glfw.window_hint(WindowHint::TransparentFramebuffer(false));
    glfw.window_hint(WindowHint::Resizable(false));

    let (mut window, events) = glfw
        .create_window(WIDTH, HEIGHT, "minecraft", WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    window.set_key_polling(true);
    window.set_size_polling(true);
    window.make_current();

    let mut gl =
        unsafe { glow::Context::from_loader_function(|s| window.get_proc_address(s) as *const _) };

    println!("OpenGL version: {}", unsafe {
        gl.get_parameter_string(glow::VERSION)
    });

    unsafe {
        gl.enable(glow::DEPTH_TEST);
        gl.enable(glow::DEBUG_OUTPUT);
        gl.debug_message_callback(|_source, _gltype, id, severity, msg| {
            println!(
                "GL CALLBACK: {} severity = {}, message = {}",
                id, severity, msg
            );
        });
    }

    unsafe { gl.viewport(0, 0, WIDTH as _, HEIGHT as _) };

    let vao = unsafe { gl.create_vertex_array().unwrap() };
    let vbo = unsafe { gl.create_buffer().unwrap() };
    let instance_vbo = unsafe { gl.create_buffer().unwrap() };
    // let ebo = unsafe { gl.create_buffer().unwrap() };

    #[rustfmt::skip]
    let instance_positions: &[f32] = &[
    //     x     y     z
         0.0,  0.0,  0.0,
         0.0,  0.0, -1.0,
         0.0,  0.0, -2.0,
         0.0,  0.0, -3.0,
         0.0,  0.0, -4.0,

         1.0,  0.0,  0.0,
         1.0,  0.0, -1.0,
         1.0,  0.0, -2.0,
         1.0,  0.0, -3.0,
         1.0,  0.0, -4.0,

         2.0,  0.0,  0.0,
         2.0,  0.0, -1.0,
         2.0,  0.0, -2.0,
         2.0,  0.0, -3.0,
         2.0,  0.0, -4.0,

         3.0,  0.0,  0.0,
         3.0,  0.0, -1.0,
         3.0,  0.0, -2.0,
         3.0,  0.0, -3.0,
         3.0,  0.0, -4.0,
    ];

    unsafe {
        gl.bind_vertex_array(Some(vao));

        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
        gl.buffer_data_u8_slice(
            glow::ARRAY_BUFFER,
            bytemuck::cast_slice(&VERTICES),
            glow::STATIC_DRAW,
        );

        // position
        gl.enable_vertex_attrib_array(0);
        gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 5 * F32S as i32, 0);

        // tex coord
        gl.enable_vertex_attrib_array(1);
        gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, 5 * F32S as i32, 3 * F32S as i32);

        gl.bind_buffer(glow::ARRAY_BUFFER, Some(instance_vbo));

        gl.buffer_data_u8_slice(
            glow::ARRAY_BUFFER,
            bytemuck::cast_slice(instance_positions),
            glow::STATIC_DRAW,
        );

        // instance position
        gl.enable_vertex_attrib_array(2);
        gl.vertex_attrib_pointer_f32(2, 3, glow::FLOAT, false, F32S as i32 * 3, 0);
        gl.vertex_attrib_divisor(2, 1);

        gl.bind_buffer(glow::ARRAY_BUFFER, None);
        gl.bind_vertex_array(None);
    }

    let shader_program = unsafe {
        let shader = parse_shader(include_str!("basic.glsl"));
        let vertex_shader = compile_shader(&gl, glow::VERTEX_SHADER, &shader["vertex"]);
        let fragment_shader = compile_shader(&gl, glow::FRAGMENT_SHADER, &shader["fragment"]);

        let program = gl.create_program().unwrap();
        gl.attach_shader(program, vertex_shader);
        gl.attach_shader(program, fragment_shader);
        gl.link_program(program);

        gl.delete_shader(vertex_shader);
        gl.delete_shader(fragment_shader);
        program
    };

    let texture_id = unsafe {
        let texture_data = load_texture("sample_texture.png");
        let id = gl.create_texture().unwrap();
        gl.bind_texture(glow::TEXTURE_2D, Some(id));

        gl.tex_image_2d(
            glow::TEXTURE_2D,
            0,
            texture_data.format as _,
            texture_data.width,
            texture_data.height,
            0,
            texture_data.format,
            glow::UNSIGNED_BYTE,
            glow::PixelUnpackData::Slice(Some(texture_data.data)),
        );

        gl.generate_mipmap(glow::TEXTURE_2D);

        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::REPEAT as _);
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::REPEAT as _);

        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MIN_FILTER,
            glow::NEAREST_MIPMAP_NEAREST as _,
        );

        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MAG_FILTER,
            glow::NEAREST as _,
        );

        id
    };

    let (width, height) = window.get_size();

    let mut camera = Camera {
        eye: (0.0, 1.0, 2.0).into(),
        target: (0.0, 0.0, 0.0).into(), // have a look at the origin
        up: glam::Vec3::Y,
        aspect: width as f32 / height as f32,
        fovy: 45.0,
        z_near: 0.1,
        z_far: 100.0,
        speed: 0.2,

        forward_pressed: false,
        backward_pressed: false,
        left_pressed: false,
        right_pressed: false,
    };

    while !window.should_close() {
        glfw.poll_events();
        glfw::flush_messages(&events).for_each(|(_, event)| {
            let window: &mut glfw::Window = &mut window;

            if camera.process_event(&event) {
                return;
            }

            match event {
                WindowEvent::Key(Key::Escape, _, Action::Press, _) => window.set_should_close(true),
                WindowEvent::Size(w, h) => {
                    camera.aspect = w as f32 / h as f32;
                    unsafe { gl.viewport(0, 0, w, h) }
                }

                _ => {}
            }
        });

        // let model = glam::Mat4::from_rotation_x(glfw.get_time() as f32 * 50.0f32.to_radians())
        // * glam::Mat4::from_rotation_y(glfw.get_time() as f32 * 50.0f32.to_radians())
        // * glam::Mat4::from_rotation_z(glfw.get_time() as f32 * 50.0f32.to_radians());

        // let model = glam::Mat4::IDENTITY;
        let view = camera.get_view();
        let projection = camera.get_projection();

        unsafe {
            gl.use_program(Some(shader_program));

            // gl.uniform_matrix_4_f32_slice(
            //     gl.get_uniform_location(shader_program, "model").as_ref(),
            //     false,
            //     &model.to_cols_array(),
            // );

            gl.uniform_matrix_4_f32_slice(
                gl.get_uniform_location(shader_program, "view").as_ref(),
                false,
                &view.to_cols_array(),
            );

            gl.uniform_matrix_4_f32_slice(
                gl.get_uniform_location(shader_program, "projection")
                    .as_ref(),
                false,
                &projection.to_cols_array(),
            );

            gl.clear_color(0.2, 0.2, 0.2, 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);

            gl.bind_texture(glow::TEXTURE_2D, Some(texture_id));
            gl.bind_vertex_array(Some(vao));
            gl.draw_arrays_instanced(glow::TRIANGLES, 0, 36, instance_positions.len() as _);
            // gl.draw_arrays(glow::TRIANGLES, 0, 36);
        }

        window.swap_buffers();
    }

    unsafe {
        gl.delete_program(shader_program);
        gl.delete_vertex_array(vao);
        gl.delete_buffer(vbo);
    }
}

struct TextureData {
    pub width: i32,
    pub height: i32,
    pub format: u32,
    pub data: &'static [u8],
}

impl Drop for TextureData {
    fn drop(&mut self) {
        unsafe {
            image::stbi_image_free(std::mem::transmute::<*const u8, *mut std::ffi::c_void>(
                self.data.as_ptr(),
            ))
        };
    }
}

fn load_texture(path: &str) -> TextureData {
    unsafe { image::stbi_set_flip_vertically_on_load(true as _) };

    let (mut width, mut height, mut channels) = (0, 0, 0);
    let data = unsafe {
        let cs = CString::new(path).unwrap();
        image::stbi_load(
            cs.as_ptr(),
            &raw mut width,
            &raw mut height,
            &raw mut channels,
            0,
        )
    };

    assert!(!data.is_null(), "Failed to load image");

    let format = match channels {
        3 => glow::RGB,
        4 => glow::RGBA,
        _ => panic!("Unsupported image format"),
    };

    TextureData {
        width,
        height,
        format,
        data: unsafe { std::slice::from_raw_parts(data, (width * height * channels) as usize) },
    }
}

fn parse_shader(input: &str) -> std::collections::HashMap<String, String> {
    let mut section = None;
    let mut code = String::new();
    let mut map = std::collections::HashMap::new();

    for line in input.lines() {
        if line.is_empty() {
            continue;
        }

        if line.starts_with("--") {
            if let Some(sec) = section.take() {
                map.insert(sec, code);
            }

            section = Some(line.trim_start_matches(['-', ' ']).to_string());
            code = String::new();
        } else {
            code.push_str(line);
            code.push('\n');
        }
    }

    if let Some(section) = section {
        map.insert(section, code);
    }

    map
}

fn compile_shader(gl: &glow::Context, shader_type: u32, source: &str) -> glow::NativeShader {
    unsafe {
        let shader = gl.create_shader(shader_type).unwrap();
        gl.shader_source(shader, source);
        gl.compile_shader(shader);

        if !gl.get_shader_compile_status(shader) {
            panic!(
                "Shader compilation failed: {}",
                gl.get_shader_info_log(shader)
            );
        }

        shader
    }
}

#[rustfmt::skip]
const VERTICES: [f32; 180] = [
    // ---------------------------
    // FRONT face (z = +0.5)
    // Using UV range u ∈ [0.50, 0.75], v ∈ [0.3333, 0.6667]

    // Triangle 1
    -0.5, -0.5, 0.5, 0.50, 0.3333, // bottom-left
    -0.5,  0.5, 0.5, 0.50, 0.6667, // top-left
     0.5,  0.5, 0.5, 0.75, 0.6667, // top-right
    // Triangle 2
    -0.5, -0.5, 0.5, 0.50, 0.3333, // bottom-left
     0.5,  0.5, 0.5, 0.75, 0.6667, // top-right
     0.5, -0.5, 0.5, 0.75, 0.3333, // bottom-right

    // ---------------------------
    // BACK face (z = -0.5)
    // Using UV range u ∈ [0.00, 0.25], v ∈ [0.3333, 0.6667]

    // Triangle 1
     0.5, -0.5, -0.5, 0.00, 0.3333,
     0.5,  0.5, -0.5, 0.00, 0.6667,
    -0.5,  0.5, -0.5, 0.25, 0.6667,
    // Triangle 2
     0.5, -0.5, -0.5, 0.00, 0.3333,
    -0.5,  0.5, -0.5, 0.25, 0.6667,
    -0.5, -0.5, -0.5, 0.25, 0.3333,

    // ---------------------------
    // LEFT face (x = -0.5)
    // Using UV range u ∈ [0.25, 0.50], v ∈ [0.3333, 0.6667]

    // Triangle 1
    -0.5, -0.5, -0.5, 0.25, 0.3333,
    -0.5,  0.5, -0.5, 0.25, 0.6667,
    -0.5,  0.5,  0.5, 0.50, 0.6667,
    // Triangle 2
    -0.5, -0.5, -0.5, 0.25, 0.3333,
    -0.5,  0.5,  0.5, 0.50, 0.6667,
    -0.5, -0.5,  0.5, 0.50, 0.3333,

    // ---------------------------
    // RIGHT face (x =  0.5)
    // Using UV range u ∈ [0.75, 1.00], v ∈ [0.3333, 0.6667]

    // Triangle 1
     0.5, -0.5,  0.5, 0.75, 0.3333,
     0.5,  0.5,  0.5, 0.75, 0.6667,
     0.5,  0.5, -0.5, 1.00, 0.6667,
    // Triangle 2
     0.5, -0.5,  0.5, 0.75, 0.3333,
     0.5,  0.5, -0.5, 1.00, 0.6667,
     0.5, -0.5, -0.5, 1.00, 0.3333,

    // ---------------------------
    // UP face (y =  0.5)
    // Using UV range u ∈ [0.25, 0.50], v ∈ [0.00, 0.3333]

    // Triangle 1
    -0.5,  0.5,  0.5, 0.25, 0.6667,
    -0.5,  0.5, -0.5, 0.25, 1.00  ,
     0.5,  0.5, -0.5, 0.50, 1.00  ,
    // Triangle 2
    -0.5,  0.5,  0.5, 0.25, 0.6667,
     0.5,  0.5, -0.5, 0.50, 1.00  ,
     0.5,  0.5,  0.5, 0.50, 0.6667,

    // ---------------------------
    // DOWN face (y = -0.5)
    // Using UV range u ∈ [0.25, 0.50], v ∈ [0.6667, 1.00]

    // Triangle 1
    -0.5, -0.5, -0.5,  0.25, 0.00  ,
    -0.5, -0.5,  0.5,  0.25, 0.3333,
     0.5, -0.5,  0.5,  0.50, 0.3333,
    // Triangle 2
    -0.5, -0.5, -0.5, 0.25, 0.00  ,
     0.5, -0.5,  0.5, 0.50, 0.3333,
     0.5, -0.5, -0.5, 0.50, 0.00  ,
];
