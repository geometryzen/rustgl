#![allow(non_upper_case_globals)]
extern crate glfw;
use self::glfw::{Action, Context, Key};

use std::str;
use std::sync::mpsc::Receiver;

mod eight;

const SCR_WIDTH: u32 = 800;
const SCR_HEIGHT: u32 = 600;

const vertexShaderSource: &str = r##"#version 330 core
    layout (location = 0) in vec3 aPos;
    void main() {
       gl_Position = vec4(aPos.x, aPos.y, aPos.z, 1.0);
    }
"##;

const fragmentShaderSource: &str = r##"#version 330 core
    out vec4 FragColor;
    void main() {
       FragColor = vec4(1.0f, 1.0f, 1.0f, 1.0f);
    }
"##;

fn clear() {
    unsafe {
        gl::ClearColor(0.1, 0.1, 0.1, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);
    }
}

fn draw() {
    unsafe {
        gl::DrawArrays(gl::TRIANGLES, 0, 3);
    }
}

pub fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(4, 6));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));
    #[cfg(target_os = "macos")]
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

    let (mut window, events) = glfw
        .create_window(SCR_WIDTH, SCR_HEIGHT, "RustGL", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window");

    window.make_current();
    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);

    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    let vertices: [f32; 9] = [
        -0.5, -0.5, 0.0, // left
        0.5, -0.5, 0.0, // right
        0.0, 0.5, 0.0, // top
    ];

    let geometry = eight::VertexArray::new(vertices);

    let material = eight::Material::new(vertexShaderSource, fragmentShaderSource);

    while !window.should_close() {
        process_events(&mut window, &events);

        clear();

        material.use_program();

        geometry.bind();

        draw();

        geometry.unbind();

        window.swap_buffers();

        glfw.poll_events();
    }
}

fn process_events(window: &mut glfw::Window, events: &Receiver<(f64, glfw::WindowEvent)>) {
    for (_, event) in glfw::flush_messages(events) {
        match event {
            glfw::WindowEvent::FramebufferSize(width, height) => {
                // make sure the viewport matches the new window dimensions; note that width and
                // height will be significantly larger than specified on retina displays.
                unsafe { gl::Viewport(0, 0, width, height) }
            }
            glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                window.set_should_close(true)
            }
            _ => {}
        }
    }
}
