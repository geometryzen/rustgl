extern crate gl;
use self::gl::types::*;

use std::mem;
use std::os::raw::c_void;
use std::ptr;

use std::ffi::CString;
use std::str;

pub struct Geometry {
    vao: GLuint,
}

impl Drop for Geometry {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &mut self.vao);
        }
    }
}

impl Geometry {
    // TODO: The parameters in this call aren't orthogonal because the data is 1:1 with the vertices.
    pub fn new(index: u32, size: i32, vertices: &[f32], indices: &[u32]) -> Geometry {
        unsafe {
            let (mut vao, mut ebo, mut vbo) = (0, 0, 0);

            gl::GenVertexArrays(1, &mut vao);

            gl::GenBuffers(1, &mut ebo);
            gl::GenBuffers(1, &mut vbo);

            let geometry = Geometry { vao };

            geometry.bind();

            // println!("indices.len() = {}", indices.len());
            // println!("mem::size_of::<GLfloat>() = {}", mem::size_of::<GLfloat>());
            // println!("mem::size_of::<GLuint>() = {}", mem::size_of::<GLuint>());
            // println!("mem::size_of::<u32>() = {}", mem::size_of::<u32>());

            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * mem::size_of::<f32>()) as GLsizeiptr,
                &vertices[0] as *const f32 as *const c_void,
                gl::STATIC_DRAW,
            );

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (indices.len() * mem::size_of::<u32>()) as GLsizeiptr,
                &indices[0] as *const u32 as *const c_void,
                gl::STATIC_DRAW,
            );

            gl::VertexAttribPointer(
                index,
                size,
                gl::FLOAT,
                gl::FALSE,
                3 * mem::size_of::<GLfloat>() as GLsizei,
                ptr::null(),
            );
            gl::EnableVertexAttribArray(index);

            // note that this is allowed, the call to gl::VertexAttribPointer registered VBO
            // as the vertex attribute's bound vertex buffer object so afterwards we can safely unbind
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);

            // remember: do NOT unbind the EBO while a VAO is active as the bound element buffer object
            // IS stored in the VAO; keep the EBO bound.
            // gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);

            geometry.unbind();

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);

            geometry
        }
    }
    pub fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.vao);
        }
    }
    pub fn unbind(&self) {
        unsafe {
            gl::BindVertexArray(0);
        }
    }
}

pub struct Material {
    program: GLuint,
}

impl Drop for Material {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.program);
        }
    }
}

impl Material {
    pub fn new(vertex_shader_source: &str, fragment_shader_source: &str) -> Material {
        // vertex shader
        unsafe {
            let vs = gl::CreateShader(gl::VERTEX_SHADER);
            let c_str_vert = CString::new(vertex_shader_source.as_bytes()).unwrap();
            gl::ShaderSource(vs, 1, &c_str_vert.as_ptr(), ptr::null());
            gl::CompileShader(vs);

            // check for shader compile errors
            let mut success = gl::FALSE as GLint;
            let capacity: usize = 512;
            let mut info_log = Vec::with_capacity(capacity);
            info_log.set_len(capacity - 1); // subtract 1 to skip the trailing null character
            gl::GetShaderiv(vs, gl::COMPILE_STATUS, &mut success);
            if success != gl::TRUE as GLint {
                gl::GetShaderInfoLog(
                    vs,
                    capacity as i32,
                    ptr::null_mut(),
                    info_log.as_mut_ptr() as *mut GLchar,
                );
                println!(
                    "ERROR::SHADER::VERTEX::COMPILATION_FAILED\n{}",
                    str::from_utf8(&info_log).unwrap()
                );
            }

            // fragment shader
            let fs = gl::CreateShader(gl::FRAGMENT_SHADER);
            let c_str_frag = CString::new(fragment_shader_source.as_bytes()).unwrap();
            gl::ShaderSource(fs, 1, &c_str_frag.as_ptr(), ptr::null());
            gl::CompileShader(fs);
            // check for shader compile errors
            gl::GetShaderiv(fs, gl::COMPILE_STATUS, &mut success);
            if success != gl::TRUE as GLint {
                gl::GetShaderInfoLog(
                    fs,
                    capacity as i32,
                    ptr::null_mut(),
                    info_log.as_mut_ptr() as *mut GLchar,
                );
                println!(
                    "ERROR::SHADER::FRAGMENT::COMPILATION_FAILED\n{}",
                    str::from_utf8(&info_log).unwrap()
                );
            }

            // link shaders
            let program = gl::CreateProgram();
            gl::AttachShader(program, vs);
            gl::AttachShader(program, fs);
            gl::LinkProgram(program);
            // check for linking errors
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
            if success != gl::TRUE as GLint {
                gl::GetProgramInfoLog(
                    program,
                    capacity as i32,
                    ptr::null_mut(),
                    info_log.as_mut_ptr() as *mut GLchar,
                );
                println!(
                    "ERROR::SHADER::PROGRAM::COMPILATION_FAILED\n{}",
                    str::from_utf8(&info_log).unwrap()
                );
            }
            gl::DeleteShader(vs);
            gl::DeleteShader(fs);

            Material { program }
        }
    }
    pub fn use_program(&self) {
        unsafe {
            gl::UseProgram(self.program);
        }
    }
}

pub struct Mesh {
    geometry: Geometry,
    material: Material,
}

impl Mesh {
    pub fn new(geometry: Geometry, material: Material) -> Mesh {
        Mesh { geometry, material }
    }
    pub fn render(&self) {
        self.material.use_program();

        self.geometry.bind();

        // TODO: Where should mode, first, and count come from?
        // draw_arrays(DrawMode::Triangles, 0, 3);
        draw_elements(DrawMode::Triangles, 6);

        self.geometry.unbind();
    }
}

pub fn clear() {
    unsafe {
        gl::ClearColor(0.1, 0.1, 0.1, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);
    }
}

#[allow(dead_code)]
#[repr(u32)]
#[derive(Clone, Copy)]
pub enum DrawMode {
    Lines = gl::LINES,
    Points = gl::POINTS,
    Triangles = gl::TRIANGLES,
}

#[allow(dead_code)]
pub fn draw_arrays(mode: DrawMode, first: GLint, count: GLsizei) {
    unsafe {
        gl::DrawArrays(mode as u32, first, count);
    }
}

#[allow(dead_code)]
#[allow(unused_variables)]
pub fn draw_elements(mode: DrawMode, count: GLsizei) {
    unsafe {
        gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());
    }
}

pub fn viewport(x: i32, y: i32, width: GLsizei, height: GLsizei) {
    unsafe { gl::Viewport(x, y, width, height) }
}

pub fn load_with<F>(load_function: F)
where
    F: FnMut(&'static str) -> *const c_void,
{
    gl::load_with(load_function);
}
