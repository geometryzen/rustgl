#![allow(non_upper_case_globals)]
extern crate glfw;
use self::glfw::{Action, Context, Key};

// extern crate gl;
// use self::gl::types::*;

use std::mem;
use std::os::raw::c_void;
use std::ptr;
use std::str;
use std::sync::mpsc::Receiver;

mod graphics;
#[allow(unused_imports)]
use graphics::{
    clear, clear_color, disable_vertex_attrib, draw_arrays, draw_elements, enable_vertex_attrib,
    vertex_attrib_pointer, viewport,
};
#[allow(unused_imports)]
use graphics::{
    Buffer, BufferTarget::Array, BufferTarget::ElementArray, DrawMode::Points, DrawMode::Triangles,
    Program, ShaderType, VertexArray,
};

// use cgmath::prelude::*;
// use cgmath::{Matrix4, Rad};

const WIDTH: u32 = 800;
const HEIGHT: u32 = 800;

// const GROUP_SIZE: u32 = 64;
// const NUM_VERTS: u32 = 256;

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

    window.set_framebuffer_size_polling(true);

    graphics::load_with(|symbol| window.get_proc_address(symbol));

    // TODO: Print out some info about the graphics drivers

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

    let va = vertex_array_from_vertices(location, size, &vertices, &indices);

    // let vbo = Buffer::new();

    let vs = ShaderType::Vertex.create();
    vs.source(vertexShaderSourceCircle);
    vs.compile().unwrap();

    let fs = ShaderType::Fragment.create();
    fs.source(fragmentShaderSourceCircle);
    fs.compile().unwrap();

    let render_program = Program::create();
    render_program.attach(&vs);
    render_program.attach(&fs);
    render_program.link().unwrap();

    // let cs = ShaderType::Compute.create();
    // cs.source(computeShaderSourceCircle);
    // cs.compile().unwrap();

    // let compute_program = Program::create();
    // compute_program.attach(&cs);
    // compute_program.link().unwrap();

    while !window.should_close() {
        process_events(&mut window, &events);

        /*
        compute_program.use_program();

        let radius_location = compute_program.get_uniform_location("radius");
        unsafe {
            // TODO: Make the radius vary for each frame.
            gl::Uniform1f(radius_location, 0.5 as GLfloat);
        }

        // Bind the VBO onto the SSBO, which is going to be filled within the compute shader.

        let index_buffer_binding = 0;
        unsafe {
            // We need a vbo object for the last argument.
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, index_buffer_binding, vbo.name);
        }

        unsafe {
            gl::DispatchCompute(NUM_VERTS / GROUP_SIZE, 1, 1);
        }

        // Unbind the SSBO buffer.
        unsafe {
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, index_buffer_binding, 0);
        }

        unsafe {
            gl::MemoryBarrier(gl::VERTEX_ATTRIB_ARRAY_BARRIER_BIT);
        }
        */

        clear_color(0.1, 0.1, 0.1, 1.0);
        clear();

        // let mut transform: Matrix4<f32> = Matrix4::identity();
        // transform = transform * Matrix4::<f32>::from_translation(vec3(0.5, -0.5, 0.0));
        // transform = transform * Matrix4::<f32>::from_angle_z(Rad(glfw.get_time() as f32));

        render_program.use_program();

        // let transform_location = render_program.get_uniform_location("transform");
        //unsafe {
        //    gl::UniformMatrix4fv(transform_location, 1, gl::FALSE, transform.as_ptr());
        //}

        va.bind();

        // Array.bind(&vbo);
        // enable_vertex_attrib(0);
        // enable_vertex_attrib(1);

        draw_elements(Triangles, 6);
        // draw_arrays(Points, 0, NUM_VERTS as GLsizei);

        va.unbind();
        // Array.unbind();

        window.swap_buffers();

        glfw.poll_events();
    }
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
                viewport(0, 0, width, height)
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

//
pub fn vertex_array_from_vertices(
    location: u32,
    size: i32,
    vertices: &[f32],
    indices: &[u32],
) -> graphics::VertexArray {
    // Create Vertex and Index buffers.
    let vb = Buffer::new();
    let ib = Buffer::new();

    // Bind the vertex buffer to the GL_ARRAY_BUFFER target.
    // This does not immediately impact the VAO. Instead this
    // buffer is noted when the attribute location  is described below.
    Array.bind(&vb);
    Array.buffer_data(
        vertices.len() * mem::size_of::<f32>(),
        &vertices[0] as *const f32 as *const c_void,
        gl::STATIC_DRAW,
    );
    Array.unbind();

    ElementArray.bind(&ib);
    ElementArray.buffer_data(
        indices.len() * mem::size_of::<u32>(),
        &indices[0] as *const u32 as *const c_void,
        gl::STATIC_DRAW,
    );
    ElementArray.unbind();

    let vao = VertexArray::new();

    vao.bind();

    // This call changes the state of the VAO.
    // https://www.khronos.org/opengl/wiki/Vertex_Specification
    // The VAO will record that the attribute with the location given will get its data
    // from the buffer that is currently bound to GL_ARRAY_BUFFER.
    // Note: If the GL_ARRAY_BUFFER has no binding then things go badly wrong.
    Array.bind(&vb);
    vertex_attrib_pointer(
        location,
        size,
        gl::FLOAT,
        gl::FALSE,
        3 * mem::size_of::<f32>(),
        ptr::null(),
    );
    Array.unbind();

    // The spec says that a new VAO has array access is disabled for all attributes.
    enable_vertex_attrib(location);

    // note that this is allowed, the call to gl::VertexAttribPointer registered VBO
    // as the vertex attribute's bound vertex buffer object so afterwards we can safely unbind
    // Array.unbind();

    // remember: do NOT unbind the EBO while a VAO is active as the bound element buffer object
    // IS stored in the VAO; keep the EBO bound.
    // Target::ElementArrayBuffer.unbind();

    // It's a bit bizarre, but the action of binding the index buffer is what is remembered by the VAO,
    // or perhaps the VAO notes the index buffer when it unbinds?
    ElementArray.bind(&ib);

    vao.unbind();

    ElementArray.unbind();

    vao
}

#[allow(dead_code)]
const vertexShaderSourceBox: &str = r##"#version 460 core
    layout (location = 0) in vec3 aPos;

    uniform mat4 transform;

    void main()
    {
       gl_Position = transform * vec4(aPos.x, aPos.y, aPos.z, 1.0);
    }
"##;

#[allow(dead_code)]
const fragmentShaderSourceBox: &str = r##"#version 460 core
    out vec4 FragColor;
    void main()
    {
       FragColor = vec4(1.0f, 1.0f, 1.0f, 1.0f);
    }
"##;

#[allow(dead_code)]
const computeShaderSourceCircle: &str = r##"#version 460 core

uniform float radius;

struct AttribData
{
    vec4 v;
    vec4 c;
};

layout(std140, binding = 0) buffer destBuffer
{
    AttribData data[];
} outBuffer;

layout (local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

void main()
{
    uint storePos = gl_GlobalInvocationID.x;

    // Calculate the global number of threads (size) for this work dispatch.
    uint gSize = gl_WorkGroupSize.x * gl_NumWorkGroups.x;

    float alpha = 2.0 * 3.14159265359 * (float(storePos) / float(gSize));

    outBuffer.data[storePos].v = vec4(sin(alpha) * radius, cos(alpha) * radius, 0.0, 1.0);

	outBuffer.data[storePos].c = vec4(float(storePos) / float(gSize), 0.0, 1.0, 1.0);
}
"##;

#[allow(dead_code)]
const vertexShaderSourceCircle: &str = r##"
attribute vec4 a_v4Position;
attribute vec4 a_v4FillColor;

varying vec4 v_v4FillColor;

void main()
{
    v_v4FillColor = a_v4FillColor;
    gl_Position = a_v4Position;
}"##;

#[allow(dead_code)]
const fragmentShaderSourceCircle: &str = r##"
varying vec4 v_v4FillColor;

void main()
{
    // gl_FragColor = v_v4FillColor;
    gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0);
}"##;
