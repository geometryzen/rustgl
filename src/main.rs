#![allow(non_upper_case_globals)]
extern crate glfw;
use self::glfw::{Action, Context, Key};

use std::str;
use std::sync::mpsc::Receiver;

mod graphics;

use cgmath::prelude::*;
use cgmath::{Matrix4, Rad};

const WIDTH: u32 = 800;
const HEIGHT: u32 = 800;

const vertexShaderSource: &str = r##"#version 460 core
    layout (location = 0) in vec3 aPos;

    uniform mat4 transform;

    void main()
    {
       gl_Position = transform * vec4(aPos.x, aPos.y, aPos.z, 1.0);
    }
"##;

const fragmentShaderSource: &str = r##"#version 460 core
    out vec4 FragColor;
    void main()
    {
       FragColor = vec4(1.0f, 0.5f, 0.2f, 1.0f);
    }
"##;

#[allow(dead_code)]
const computeShaderSource: &str = r##"#version 310 es

// The uniform parameters which is passed from application for every frame.

uniform float radius;

// Declare custom data struct, which represents either vertex or colour.

struct Vector3f

{

      float x;

      float y;

      float z;

      float w;

};

// Declare the custom data type, which represents one point of a circle.

// And this is vertex position and colour respectively.

// As you may already noticed that will define the interleaved data within

// buffer which is Vertex|Colour|Vertex|Colour|…

struct AttribData

{

      Vector3f v;

      Vector3f c;

};

// Declare input/output buffer from/to wich we will read/write data.

// In this particular shader we only write data into the buffer.

// If you do not want your data to be aligned by compiler try to use:

// packed or shared instead of std140 keyword.

// We also bind the buffer to index 0. You need to set the buffer binding

// in the range [0..3] – this is the minimum range approved by Khronos.

// Notice that various platforms might support more indices than that.

layout(std140, binding = 0) buffer destBuffer

{

      AttribData data[];

} outBuffer;

// Declare what size is the group. In our case is 8x8, which gives

// 64 group size.

layout (local_size_x = 8, local_size_y = 8, local_size_z = 1) in;

// Declare main program function which is executed once

// glDispatchCompute is called from the application.

void main()

{

      // Read current global position for this thread

      ivec2 storePos = ivec2(gl_GlobalInvocationID.xy);

      // Calculate the global number of threads (size) for this

      uint gWidth = gl_WorkGroupSize.x * gl_NumWorkGroups.x;

      uint gHeight = gl_WorkGroupSize.y * gl_NumWorkGroups.y;

      uint gSize = gWidth * gHeight;

      // Since we have 1D array we need to calculate offset.

      uint offset = storePos.y * gWidth + storePos.x;

      // Calculate an angle for the current thread

      float alpha = 2.0 * 3.14159265359 * (float(offset) / float(gSize));

      // Calculate vertex position based on the already calculate angle

      // and radius, which is given by application

      outBuffer.data[offset].v.x = sin(alpha) * radius;

      outBuffer.data[offset].v.y = cos(alpha) * radius;

      outBuffer.data[offset].v.z = 0.0;

      outBuffer.data[offset].v.w = 1.0;

      // Assign colour for the vertex

      outBuffer.data[offset].c.x = storePos.x / float(gWidth);

      outBuffer.data[offset].c.y = 0.0;

      outBuffer.data[offset].c.z = 1.0;

      outBuffer.data[offset].c.w = 1.0;

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

    graphics::load_with(|symbol| window.get_proc_address(symbol));

    let vertices: [f32; 24] = [
        0.5, 0.5, 0.5, -0.5, 0.5, 0.5, -0.5, -0.5, 0.5, 0.5, -0.5, 0.5, 0.5, -0.5, -0.5, 0.5, 0.5,
        -0.5, -0.5, 0.5, -0.5, -0.5, -0.5, -0.5,
    ];

    let indices: [u32; 36] = [
        1, 2, 0, 3, 0, 2, 5, 6, 0, 1, 0, 6, 2, 1, 7, 6, 7, 1, 5, 0, 4, 3, 4, 0, 2, 7, 3, 4, 3, 7,
        7, 6, 4, 6, 4, 6,
    ];

    let location = 0;
    let size = 3;

    let vao = graphics::VertexArray::new(location, size, &vertices, &indices);

    let pgm = graphics::Program::new(vertexShaderSource, fragmentShaderSource);

    while !window.should_close() {
        process_events(&mut window, &events);

        graphics::clear();

        let mut transform: Matrix4<f32> = Matrix4::identity();
        // transform = transform * Matrix4::<f32>::from_translation(vec3(0.5, -0.5, 0.0));
        transform = transform * Matrix4::<f32>::from_angle_z(Rad(glfw.get_time() as f32));

        pgm.use_program();

        let location = pgm.get_uniform_location("transform");
        unsafe {
            gl::UniformMatrix4fv(location, 1, gl::FALSE, transform.as_ptr());
        }

        vao.bind();

        graphics::draw_elements(graphics::DrawMode::Triangles, 6);

        vao.unbind();

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
                graphics::viewport(0, 0, width, height)
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
