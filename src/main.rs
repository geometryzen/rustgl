#![allow(non_upper_case_globals)]
extern crate glfw;
use self::glfw::{Action, Context, Key};

use std::str;
use std::sync::mpsc::Receiver;

mod eight;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 800;

const vertexShaderSource: &str = r##"#version 460 core
    layout (location = 0) in vec3 aPos;
    void main()
    {
       gl_Position = vec4(aPos.x, aPos.y, aPos.z, 1.0);
    }
"##;

const fragmentShaderSource: &str = r##"#version 460 core
    out vec4 FragColor;
    void main()
    {
       FragColor = vec4(1.0f, 1.0f, 1.0f, 1.0f);
    }
"##;

pub fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(4, 6));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));
    #[cfg(target_os = "macos")]
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

    let (mut window, events) = glfw
        .create_window(WIDTH, HEIGHT, "RustGL", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window");

    window.make_current();
    window.set_key_polling(true);

    // TODO: I don't think the documentation on this is correct.
    window.set_framebuffer_size_polling(true);

    eight::load_with(|symbol| window.get_proc_address(symbol));

    let vertices: [f32; 9] = [
        -0.5, -0.5, 0.0, // left
        0.5, -0.5, 0.0, // right
        0.0, 0.5, 0.0, // top
    ];

    let location = 0;
    let size = 3;
    let geometry = eight::Geometry::new(location, size, &vertices);

    let material = eight::Material::new(vertexShaderSource, fragmentShaderSource);

    let mesh = eight::Mesh::new(geometry, material);

    while !window.should_close() {
        process_events(&mut window, &events);

        eight::clear();

        mesh.render();

        window.swap_buffers();

        glfw.poll_events();
    }

    // glfwTerminate is not available. It's probably part of the Drop implementation for glfw::Glfw.
}

/// This function is called directly from inside the rendering loop.
///
/// We could have used a callback with GLFW but this would make synchronization with rendering more difficult.
fn process_events(window: &mut glfw::Window, events: &Receiver<(f64, glfw::WindowEvent)>) {
    // There's spome clever stuff going on here with channels.
    for (_, event) in glfw::flush_messages(events) {
        match event {
            glfw::WindowEvent::FramebufferSize(width, height) => {
                // make sure the viewport matches the new window dimensions; note that width and
                // height will be significantly larger than specified on retina displays.

                // glViewport sets up the transformation from gl_Position values in the vertex shader
                // output to the window. gl_Position values are between -1 and 1.
                eight::viewport(0, 0, width, height)
            }
            glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                window.set_should_close(true)
            }
            glfw::WindowEvent::Key(Key::Num1, _, Action::Press, _) => unsafe {
                gl::PolygonMode(gl::FRONT_AND_BACK, gl::POINT)
            },
            glfw::WindowEvent::Key(Key::Num2, _, Action::Press, _) => unsafe {
                gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE)
            },
            glfw::WindowEvent::Key(Key::Num3, _, Action::Press, _) => unsafe {
                gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL)
            },
            _ => {}
        }
    }
}
