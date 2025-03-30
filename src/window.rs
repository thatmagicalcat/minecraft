use glfw::*;

use super::{HEIGHT, WIDTH};

pub struct CreateWindowOutput {
    pub window: PWindow,
    pub events: GlfwReceiver<(f64, WindowEvent)>,
    pub glfw: Glfw,
}

pub fn create_window() -> CreateWindowOutput {
    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();

    // OpenGL 3.3 core
    glfw.window_hint(WindowHint::ContextVersionMinor(3));
    glfw.window_hint(WindowHint::ContextVersionMajor(3));
    glfw.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));

    #[cfg(target_os = "macos")]
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

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

    CreateWindowOutput {
        window,
        events,
        glfw,
    }
}

pub fn load_gl(window: &mut PWindow) -> glow::Context {
    unsafe { glow::Context::from_loader_function(|s| window.get_proc_address(s) as *const _) }
}
