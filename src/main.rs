use std::time::Instant;

use glfw::*;
use glow::*;

mod defer;
mod renderer;
mod window;

// use camera::*;

const WIDTH: u32 = 1000;
const HEIGHT: u32 = 1000;

fn main() {
    let window::CreateWindowOutput {
        mut window,
        events,
        mut glfw,
    } = window::create_window();

    let mut gl = window::load_gl(&mut window);

    println!("OpenGL {}", unsafe {
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

    // Vec::with_capacity(chunk_size.0 * chunk_size.1 * chunk_size.2);
    let chunk_size = (4, 4, 4); // (length, breadth, depth)
    let mut instance_positions: Vec<f32> = vec![];
    let spacing = 1.0;

    for chunk_x in 0..2 {
        // for chunk_y in 0..2 {
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
        // }
    }

    let (width, height) = window.get_size();

    let mut keyboard_state = renderer::KeyboardState::default();
    let mut pointer_state = renderer::PointerState::default();

    let light_position = glam::vec3(-2.0, 8.0, -5.0);
    let light_color = glam::vec3(1.0, 1.0, 1.0);

    let mut clock = Instant::now();
    let mut click_start_position = None;

    let mut renderer = renderer::Renderer::new(
        &gl,
        renderer::Camera::new(
            (0.0, 6.0, 3.0).into(),
            45_f32.to_radians(),
            0.1,
            100.0,
            width as _,
            height as _,
        ),
        &instance_positions,
        light_color,
        light_position,
    );

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

                // window resize event
                WindowEvent::Size(w, h) => {
                    renderer.resize_camera(w as _, h as _);
                    unsafe { gl.viewport(0, 0, w, h) }
                }

                _ => {}
            }
        });

        renderer.update(dt, pointer_state, keyboard_state);

        if pointer_state.secondary_down {
            window.set_cursor_mode(CursorMode::Hidden);
        } else {
            window.set_cursor_mode(CursorMode::Normal);
        }

        if !pointer_state.secondary_down && click_start_position.is_some() {
            let glam::DVec2 { x, y } = click_start_position.take().unwrap();
            window.set_cursor_pos(x, y);
        }

        unsafe {
            gl.clear_color(0.2, 0.2, 0.2, 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);

            renderer.render();
        }

        window.swap_buffers();
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
