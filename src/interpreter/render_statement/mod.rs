use crate::interpreter::error::{InterpreterError, Result};
use crate::interpreter::render_statement::buffers_data::BuffersData;
use crate::interpreter::render_statement::pipeline_data::PipelineData;
use crate::interpreter::render_statement::shader_generator::ShaderGenerator;
use crate::interpreter::render_statement::vertex::{Vertex, create_vertex_buffer};
use glium::glutin::surface::WindowSurface;
use glium::{Display, Program, VertexBuffer};
use std::collections::HashMap;

pub mod buffers_data;
pub mod pipeline_data;
pub mod shader_generator;
pub mod vertex;

pub struct RenderStatement {
    pub vertex_shader: String,
    pub fragment_shader: String,
    pub vertex_buffer: VertexBuffer<Vertex>,
}

impl RenderStatement {
    pub fn new(
        display: &Display<WindowSurface>,
        pipeline_data: PipelineData,
        buffers_data: BuffersData,
    ) -> Result<Self> {
        Ok(Self {
            vertex_shader: ShaderGenerator::generate_vertex_shader(
                &pipeline_data.attributes,
                &HashMap::new(),
            ),
            fragment_shader: ShaderGenerator::generate_fragment_shader(
                &pipeline_data.attributes,
                &HashMap::new(),
            ),
            vertex_buffer: create_vertex_buffer(
                display,
                &buffers_data.data[..],
                &buffers_data.layout,
                &pipeline_data.attributes,
            )?,
        })
    }

    pub fn build_program(&self, display: &Display<WindowSurface>) -> Result<Program> {
        Program::from_source(display, &self.vertex_shader, &self.fragment_shader, None)
            .map_err(|e| InterpreterError::Custom(e.to_string()))
    }
}
