use std::{ffi::CString, time::Instant};

use glfw::*;
use glow::*;

mod camera;
use camera::*;

use stb_image::stb_image as image;

const WIDTH: u32 = 1900;
const HEIGHT: u32 = 1000;

const F32S: usize = std::mem::size_of::<f32>();

fn main() {
    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();

    glfw.window_hint(WindowHint::ContextVersionMinor(3));
    glfw.window_hint(WindowHint::ContextVersionMajor(3));
    glfw.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));
    glfw.window_hint(WindowHint::Decorated(false)); // No frame
    glfw.window_hint(WindowHint::TransparentFramebuffer(false));
    glfw.window_hint(WindowHint::Resizable(true));

    let (mut window, events) = glfw
        .create_window(WIDTH, HEIGHT, "minecraft", WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    window.set_key_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_mouse_button_polling(true);
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
        gl.enable(glow::CULL_FACE);
        gl.cull_face(glow::FRONT);

        gl.debug_message_callback(|_source, _gltype, id, severity, msg| {
            println!(
                "GL CALLBACK: {} severity = {}, message = {}",
                id, severity, msg
            );
        });
    }

    unsafe { gl.viewport(0, 0, WIDTH as _, HEIGHT as _) };

    let vao = unsafe { gl.create_vertex_array().unwrap() };
    let cube_vbo = unsafe { gl.create_buffer().unwrap() };
    let instance_vbo = unsafe { gl.create_buffer().unwrap() };

    // Vec::with_capacity(chunk_size.0 * chunk_size.1 * chunk_size.2);
    let chunk_size = (4, 4, 4); // (length, breadth, depth)
    let mut instance_positions: Vec<f32> = vec![];
    let spacing = 1.0;

    for chunk_x in 0..2 {
        for chunk_y in 0..2 {
            for chunk_z in 0..2 {
                let chunk_data = generate_chunk(
                    chunk_x,
                    0,
                    // chunk_y,
                    chunk_z,
                    chunk_size.0,
                    chunk_size.1,
                    chunk_size.2,
                    spacing,
                );

                instance_positions.extend(chunk_data);
            }
        }
    }

    unsafe {
        gl.bind_vertex_array(Some(vao));

        gl.bind_buffer(glow::ARRAY_BUFFER, Some(cube_vbo));
        gl.buffer_data_u8_slice(
            glow::ARRAY_BUFFER,
            bytemuck::cast_slice(&VERTICES),
            glow::STATIC_DRAW,
        );

        let stride = (8 * F32S) as i32;

        // position
        gl.enable_vertex_attrib_array(0);
        gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, stride, 0);

        // tex coord
        gl.enable_vertex_attrib_array(1);
        gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, stride, 3 * F32S as i32);

        // normal
        gl.enable_vertex_attrib_array(2);
        gl.vertex_attrib_pointer_f32(2, 3, glow::FLOAT, false, stride, 5 * F32S as i32);

        gl.bind_buffer(glow::ARRAY_BUFFER, Some(instance_vbo));

        gl.buffer_data_u8_slice(
            glow::ARRAY_BUFFER,
            bytemuck::cast_slice(&instance_positions),
            glow::STATIC_DRAW,
        );

        // instance position
        gl.enable_vertex_attrib_array(3);
        gl.vertex_attrib_pointer_f32(3, 3, glow::FLOAT, false, F32S as i32 * 3, 0);
        gl.vertex_attrib_divisor(3, 1);

        gl.bind_buffer(glow::ARRAY_BUFFER, None);
        gl.bind_vertex_array(None);
    }

    // lighting

    let light_vao = unsafe { gl.create_vertex_array().unwrap() };
    unsafe {
        gl.bind_vertex_array(Some(light_vao));
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(cube_vbo));

        // position
        gl.enable_vertex_attrib_array(0);
        gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 5 * F32S as i32, 0);
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
        let tex_dirt = load_texture("res/dirt.png");

        let id = gl.create_texture().unwrap();
        gl.bind_texture(glow::TEXTURE_2D, Some(id));

        gl.tex_image_2d(
            glow::TEXTURE_2D,
            0,
            tex_dirt.format as _,
            tex_dirt.width,
            tex_dirt.height,
            0,
            tex_dirt.format,
            glow::UNSIGNED_BYTE,
            glow::PixelUnpackData::Slice(Some(tex_dirt.data)),
        );

        gl.generate_mipmap(glow::TEXTURE_2D);

        setup_texutre_params(&gl, glow::TEXTURE_2D);

        id
    };

    let (width, height) = window.get_size();

    let mut camera = Camera::new(
        (0.0, 6.0, 3.0).into(),
        45_f32.to_radians(),
        0.1,
        100.0,
        width as _,
        height as _,
    );

    let mut keyboard_state = KeyboardState::default();
    let mut pointer_state = PointerState::default();

    let light_position = glam::vec3(-2.0, 8.0, -5.0);
    let light_color = glam::vec3(1.0, 1.0, 1.0);

    let mut clock = Instant::now();
    let mut click_start_position = None;

    while !window.should_close() {
        let dt = clock.elapsed().as_nanos() as f32 / 1e9;
        clock = Instant::now();

        glfw.poll_events();
        glfw::flush_messages(&events).for_each(|(_, event)| {
            let window: &mut glfw::Window = &mut window;

            match event {
                WindowEvent::Key(Key::Escape, _, Action::Press, _) => window.set_should_close(true),

                WindowEvent::Key(key, _, action, _) => {
                    let value = matches!(action, Action::Press | Action::Repeat);
                    match key {
                        Key::W => keyboard_state.w = value,
                        Key::A => keyboard_state.a = value,
                        Key::S => keyboard_state.s = value,
                        Key::D => keyboard_state.d = value,
                        Key::Q => keyboard_state.q = value,
                        Key::E => keyboard_state.e = value,

                        _ => {}
                    }
                }

                WindowEvent::MouseButton(btn, action, _) => {
                    let value = matches!(action, Action::Press);
                    if matches!(btn, MouseButtonRight) {
                        pointer_state.secondary_down = value;
                        click_start_position = Some(glam::DVec2::from(window.get_cursor_pos()));
                    }
                }

                WindowEvent::CursorPos(x, y) => {
                    pointer_state.pos = Some(glam::vec2(x as _, y as _));
                }

                WindowEvent::Size(w, h) => {
                    camera.resize(w as _, h as _);
                    unsafe { gl.viewport(0, 0, w, h) }
                }

                _ => {}
            }
        });

        camera.update(dt, pointer_state, keyboard_state);

        if pointer_state.secondary_down {
            window.set_cursor_mode(CursorMode::Hidden);
        } else {
            window.set_cursor_mode(CursorMode::Normal);
        }

        if !pointer_state.secondary_down && click_start_position.is_some() {
            let glam::DVec2 { x, y } = click_start_position.take().unwrap();
            window.set_cursor_pos(x, y);
        }

        camera.recalculate_view();
        camera.recalculate_projection();

        let view = camera.get_view();
        let projection = camera.get_projection();

        unsafe {
            gl.use_program(Some(shader_program));

            // eye position
            gl.uniform_3_f32(
                gl.get_uniform_location(shader_program, "eye_position")
                    .as_ref(),
                camera.get_position().x,
                camera.get_position().y,
                camera.get_position().z,
            );

            // light position
            gl.uniform_3_f32(
                gl.get_uniform_location(shader_program, "light_position")
                    .as_ref(),
                light_position.x,
                light_position.y,
                light_position.z,
            );

            // light color
            gl.uniform_3_f32(
                gl.get_uniform_location(shader_program, "light_color")
                    .as_ref(),
                light_color.x,
                light_color.y,
                light_color.z,
            );

            // view matrix
            gl.uniform_matrix_4_f32_slice(
                gl.get_uniform_location(shader_program, "view").as_ref(),
                false,
                &view.to_cols_array(),
            );

            // projection matrix
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
        }

        window.swap_buffers();
    }

    unsafe {
        gl.delete_program(shader_program);
        gl.delete_vertex_array(vao);
        gl.delete_buffer(cube_vbo);
    }
}

fn setup_texutre_params(gl: &glow::Context, target: u32) {
    unsafe {
        gl.tex_parameter_i32(target, glow::TEXTURE_WRAP_S, glow::REPEAT as _);
        gl.tex_parameter_i32(target, glow::TEXTURE_WRAP_T, glow::REPEAT as _);

        gl.tex_parameter_i32(
            target,
            glow::TEXTURE_MIN_FILTER,
            glow::NEAREST_MIPMAP_LINEAR as _,
        );

        gl.tex_parameter_i32(target, glow::TEXTURE_MAG_FILTER, glow::NEAREST as _);
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

fn generate_chunk(
    chunk_x: i32,
    chunk_y: i32,
    chunk_z: i32,
    length: i32,
    breadth: i32,
    depth: i32,
    spacing: f32,
) -> Vec<f32> {
    let mut positions = Vec::with_capacity((length * breadth * depth) as usize);

    for x in 0..breadth {
        for y in 0..length {
            for z in 0..depth {
                let world_x = chunk_x as f32 * length as f32 * spacing + x as f32 * spacing;
                let world_y = chunk_y as f32 * breadth as f32 * spacing + y as f32 * spacing;
                let world_z = chunk_z as f32 * depth as f32 * spacing - z as f32 * spacing; // Negative Z direction

                positions.extend_from_slice(&[world_x, world_y, world_z]);
            }
        }
    }

    positions
}

/// x, y, z, s, t, nx, ny, nz
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
