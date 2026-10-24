use std::{fs, ptr};
use glfw::Context;
use gl::{FRONT_AND_BACK, LINE, types::*};
use std::ffi::CString;

mod arthropods;

use arthropods::{Transform, Mesh, Model};

fn compile_shader(kind: GLenum, source: &str) -> Result<GLuint, String> {
    let c_src = CString::new(source).map_err(|e| e.to_string())?;

    unsafe {
        let shader = gl::CreateShader(kind);
        gl::ShaderSource(shader, 1, &c_src.as_ptr(), ptr::null());
        gl::CompileShader(shader);

        let mut success: GLint = 0;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
        if success == 0 {
            let mut len: GLint = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);

            let mut buf = vec![0u8; len.max(1) as usize];
            let mut written: GLsizei = 0;
            gl::GetShaderInfoLog(shader, len, &mut written, buf.as_mut_ptr() as *mut GLchar);
            buf.truncate(written as usize);

            gl::DeleteShader(shader);
            return Err(String::from_utf8_lossy(&buf).into_owned());
        }
        Ok(shader)
    }
}

fn link_program(vertex: GLuint, fragment: GLuint) -> Result<GLuint, String> {
    unsafe {
        let program = gl::CreateProgram();
        gl::AttachShader(program, vertex);
        gl::AttachShader(program, fragment);
        gl::LinkProgram(program);

        let mut success: GLint = 0;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
        if success == 0 {
            let mut len: GLint = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);

            let mut buf = vec![0u8; len.max(1) as usize];
            let mut written: GLsizei = 0;
            gl::GetProgramInfoLog(program, len, &mut written, buf.as_mut_ptr() as *mut GLchar);
            buf.truncate(written as usize);

            gl::DeleteProgram(program);
            return Err(String::from_utf8_lossy(&buf).into_owned());
        }
        Ok(program)
    }
}

fn create_shader_program(vertex_path: &str, fragment_path: &str) -> Result<GLuint, String> {
    let vertex_src = fs::read_to_string(vertex_path)
        .map_err(|e| format!("{vertex_path}: {e}"))?;
    let fragment_src = fs::read_to_string(fragment_path)
        .map_err(|e| format!("{fragment_path}: {e}"))?;

    if vertex_src.is_empty() || fragment_src.is_empty() {
        return Err("empty shader source".to_string());
    }

    let vertex_shader = compile_shader(gl::VERTEX_SHADER, &vertex_src)
        .map_err(|e| format!("vertex compilation error:\n{e}"))?;

    let fragment_shader = match compile_shader(gl::FRAGMENT_SHADER, &fragment_src) {
        Ok(s) => s,
        Err(e) => {
            unsafe { gl:: DeleteShader(vertex_shader) };
            return Err(format!("fragment compilation error:\n{e}"));
        }
    };

    let program = link_program(vertex_shader, fragment_shader)
        .map_err(|e| format!("linking error:\n{e}"));

    unsafe {
        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);
    }

    program
}

fn main() {
    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));

    let (mut window, _events) = glfw.create_window(800, 600, "", glfw::WindowMode::Windowed).unwrap();
    window.make_current();
    gl::load_with(|s| {
        window.get_proc_address(s).map_or(std::ptr::null(), |f| f as *const _)
    });

    unsafe { gl::Enable(gl::DEPTH_TEST) }; 

    let shader_program = match create_shader_program("vertex_shader.glsl", "fragment_shader.glsl") {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{e}");
            return;
        }
    };

    let model_name = CString::new("model").unwrap();
    let model_loc: GLint = unsafe { gl::GetUniformLocation(shader_program, model_name.as_ptr()) };
    let view_name = CString::new("view").unwrap();
    let view_loc: GLint = unsafe { gl::GetUniformLocation(shader_program, view_name.as_ptr()) };
    let projection_name = CString::new("projection").unwrap();
    let projection_loc: GLint = unsafe { gl::GetUniformLocation(shader_program, projection_name.as_ptr()) };

    unsafe { gl::PolygonMode(FRONT_AND_BACK, LINE) };

    while !window.should_close() {
        unsafe {
            gl::ClearColor(0.133 ,0.141, 0.212, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::UseProgram(shader_program);
        }
        window.swap_buffers();
        glfw.poll_events();
    }
}
