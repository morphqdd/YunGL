use crate::interpreter::error::{InterpreterError, Result};
use crate::interpreter::render_statement::uniform_generator::UniformValueWrapper;
use crate::interpreter::render_statement::vertex::Vertex;
use glium::glutin::surface::WindowSurface;
use glium::index::PrimitiveType;
use glium::{Display, Program, VertexBuffer};
use std::collections::HashMap;

pub mod buffers_data;
pub mod pipeline_data;
pub mod shader_generator;
pub mod uniform_generator;
pub mod vertex;

pub struct RenderStatement {
    pub vertex_shader: String,
    pub fragment_shader: String,
    pub vertex_buffer: VertexBuffer<Vertex>,
    pub uniforms: HashMap<String, UniformValueWrapper>,
    pub primitive_type: PrimitiveType,
    pub light_names: Vec<String>,
}

impl RenderStatement {
    pub fn build_program(&self, display: &Display<WindowSurface>) -> Result<Program> {
        Program::from_source(display, &self.vertex_shader, &self.fragment_shader, None)
            .map_err(|e| InterpreterError::Custom(e.to_string()))
    }
}
