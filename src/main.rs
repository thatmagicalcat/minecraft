use std::time::Instant;

use glfw::*;
use glow::*;

mod defer;
mod renderer;
mod window;

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

    let mut instance_positions: Vec<f32> = vec![];
    let mut instance_texture_ids: Vec<i32> = vec![];

    for block_x in 0..20 {
        for block_y in 0..20 {
            for block_z in 0..20 {
                instance_texture_ids.push(if block_x == 0 { 0 } else { 1 });
                instance_positions.extend_from_slice(&[
                    block_y as f32,
                    -block_x as f32,
                    block_z as f32,
                ]);
            }
        }
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
            (0.0, 2.0, 10.0).into(),
            45_f32.to_radians(),
            0.1,
            100.0,
            width as _,
            height as _,
        ),
        &instance_positions,
        &instance_texture_ids,
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

                        Key::LeftShift => keyboard_state.shift = value,

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
