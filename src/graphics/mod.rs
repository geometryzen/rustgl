extern crate gl;
use self::gl::types::*;

use std::os::raw::c_void;
use std::ptr;

use std::ffi::CStr;
use std::ffi::CString;
use std::str;

/// A BufferTarget is something to which a Buffer object may be bound.
#[allow(dead_code)]
pub enum BufferTarget {
    Array,
    ElementArray,
    ShaderStorage,
}

impl BufferTarget {
    pub fn bind(self, buffer: &Buffer) {
        unsafe {
            gl::BindBuffer(self.target(), buffer.name);
        }
    }
    pub fn unbind(self) -> () {
        unsafe {
            gl::BindBuffer(self.target(), 0);
        }
    }
    pub fn buffer_data(self, size: usize, data: *const c_void, usage: GLenum) {
        unsafe {
            gl::BufferData(self.target(), size as GLsizeiptr, data, usage);
        }
    }
    fn target(self) -> GLenum {
        match self {
            BufferTarget::Array => gl::ARRAY_BUFFER,
            BufferTarget::ElementArray => gl::ELEMENT_ARRAY_BUFFER,
            BufferTarget::ShaderStorage => gl::SHADER_STORAGE_BUFFER,
        }
    }
}

/// Buffers can be bound to multiple targets, which implies they should be handled generically.
pub struct Buffer {
    name: GLuint,
}

impl Buffer {
    pub fn new() -> Buffer {
        let mut name = 0;
        unsafe {
            // generate buffer object name.
            gl::GenBuffers(1, &mut name);
        }
        Buffer { name }
    }
}

// TODO: Why is it OK to delete an buffer, but still use it in a VertexArray?
impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            // delete the buffer object and allow the name to be reused.
            // println!("drop Buffer[name={}]", self.name);
            gl::DeleteBuffers(1, &mut self.name);
        }
    }
}

pub struct VertexArray {
    name: GLuint,
}

impl Drop for VertexArray {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &mut self.name);
        }
    }
}

impl VertexArray {
    pub fn new() -> VertexArray {
        let mut name = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut name);
        }
        VertexArray { name }
    }
    /// glBindVertexArray(self.ID)
    pub fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.name);
        }
    }
    /// glBindVertexArray(0)
    pub fn unbind(&self) {
        unsafe {
            gl::BindVertexArray(0);
        }
    }
}

#[allow(dead_code)]
pub enum ShaderType {
    Vertex,
    Fragment,
    Compute,
    Geometry,
}

impl ShaderType {
    pub fn create(self) -> Shader {
        let id = unsafe { gl::CreateShader(self.target()) };
        Shader { id }
    }
    fn target(self) -> GLenum {
        match self {
            ShaderType::Vertex => gl::VERTEX_SHADER,
            ShaderType::Fragment => gl::FRAGMENT_SHADER,
            ShaderType::Compute => gl::COMPUTE_SHADER,
            ShaderType::Geometry => gl::GEOMETRY_SHADER,
        }
    }
}

pub struct Shader {
    id: GLuint,
}

impl Shader {
    pub fn source(&self, text: &str) {
        unsafe {
            let c_str_vert = CString::new(text.as_bytes()).unwrap();
            gl::ShaderSource(self.id, 1, &c_str_vert.as_ptr(), ptr::null());
        }
    }
    pub fn compile(&self) -> Result<(), String> {
        unsafe {
            gl::CompileShader(self.id);

            let mut status: GLint = 0;
            gl::GetShaderiv(self.id, gl::COMPILE_STATUS, &mut status);
            if status == gl::TRUE as GLint {
                Ok(())
            } else {
                Err(self.info_log())
            }
        }
    }
    fn info_log(&self) -> String {
        let mut log_length: GLint = 0;
        unsafe {
            gl::GetShaderiv(self.id, gl::INFO_LOG_LENGTH, &mut log_length);
            if log_length > 0 {
                let capacity = log_length as usize;
                let mut info_log: Vec<u8> = Vec::with_capacity(capacity);
                info_log.set_len(capacity);

                gl::GetShaderInfoLog(
                    self.id,
                    log_length,
                    ptr::null_mut(),
                    info_log.as_mut_ptr() as *mut GLchar,
                );
                CStr::from_bytes_with_nul(&info_log)
                    .unwrap()
                    .to_owned()
                    .into_string()
                    .unwrap()
            } else {
                String::new()
            }
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.id);
        }
    }
}

pub struct Program {
    id: GLuint,
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}

impl Program {
    pub fn create() -> Program {
        let id = unsafe { gl::CreateProgram() };
        Program { id }
    }
    pub fn attach(&self, shader: &Shader) {
        unsafe {
            gl::AttachShader(self.id, shader.id);
        }
    }
    pub fn link(&self) -> Result<(), String> {
        unsafe {
            gl::LinkProgram(self.id);

            let mut status: GLint = 0;
            gl::GetProgramiv(self.id, gl::LINK_STATUS, &mut status);
            if status == gl::TRUE as GLint {
                Ok(())
            } else {
                Err(self.info_log())
            }
        }
    }
    fn info_log(&self) -> String {
        let mut log_length: GLint = 0;
        unsafe {
            gl::GetProgramiv(self.id, gl::INFO_LOG_LENGTH, &mut log_length);
            if log_length > 0 {
                let capacity = log_length as usize;
                let mut info_log: Vec<u8> = Vec::with_capacity(capacity);
                info_log.set_len(capacity);

                gl::GetProgramInfoLog(
                    self.id,
                    log_length,
                    ptr::null_mut(),
                    info_log.as_mut_ptr() as *mut GLchar,
                );
                CStr::from_bytes_with_nul(&info_log)
                    .unwrap()
                    .to_owned()
                    .into_string()
                    .unwrap()
            } else {
                String::new()
            }
        }
    }
    pub fn use_program(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }
    pub fn get_uniform_location(&self, name: &str) -> i32 {
        unsafe { gl::GetUniformLocation(self.id, CString::new(name).unwrap().as_ptr()) }
    }
}

#[allow(dead_code)]
pub fn clear_color(red: f32, green: f32, blue: f32, alpha: f32) {
    unsafe {
        gl::ClearColor(red, green, blue, alpha);
    }
}

pub fn clear() {
    unsafe {
        gl::Clear(gl::COLOR_BUFFER_BIT);
    }
}

pub fn vertex_attrib_pointer(
    index: GLuint,
    size: GLint,
    type_: GLenum,
    normalized: GLboolean,
    stride: usize,
    pointer: *const GLvoid,
) {
    unsafe {
        gl::VertexAttribPointer(index, size, type_, normalized, stride as GLsizei, pointer);
    }
}

pub fn enable_vertex_attrib_array(index: GLuint) {
    unsafe {
        gl::EnableVertexAttribArray(index);
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
