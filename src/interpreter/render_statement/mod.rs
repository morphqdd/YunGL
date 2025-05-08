use crate::interpreter::error::{InterpreterError, Result};
use crate::interpreter::render_statement::buffers_data::BuffersData;
use crate::interpreter::render_statement::pipeline_data::PipelineData;
use crate::interpreter::render_statement::shader_generator::ShaderGenerator;
use crate::interpreter::render_statement::uniform_generator::UniformValueWrapper;
use crate::interpreter::render_statement::vertex::{Vertex, create_vertex_buffer};
use glium::glutin::surface::WindowSurface;
use glium::index::PrimitiveType;
use glium::uniforms::{EmptyUniforms, UniformType, UniformsStorage};
use glium::{Display, Program, VertexBuffer, uniform};
use std::collections::HashMap;

const DEFAULT_MATRIX: [[f32; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

pub mod buffers_data;
pub mod pipeline_data;
pub mod shader_generator;
pub mod uniform_generator;
pub mod vertex;

pub type UniformStore<'a> = UniformsStorage<
    'a,
    [f32; 3],
    UniformsStorage<
        'a,
        [[f32; 4]; 4],
        UniformsStorage<
            'a,
            [[f32; 4]; 4],
            UniformsStorage<
                'a,
                [[f32; 4]; 4],
                UniformsStorage<'a, [f32; 3], UniformsStorage<'a, f32, EmptyUniforms>>,
            >,
        >,
    >,
>;

pub struct RenderStatement {
    pub vertex_shader: String,
    pub fragment_shader: String,
    pub vertex_buffer: VertexBuffer<Vertex>,
    pub uniforms: UniformStore<'static>,
    pub primitive_type: PrimitiveType,
}

impl RenderStatement {
    pub fn new(
        display: &Display<WindowSurface>,
        pipeline_data: PipelineData,
        buffers_data: BuffersData,
        primitive: String,
    ) -> Result<Self> {
        let uniform_values = pipeline_data.uniforms;
        Ok(Self {
            vertex_shader: ShaderGenerator::generate_vertex_shader(
                &pipeline_data.attributes,
                &uniform_values,
            ),
            fragment_shader: ShaderGenerator::generate_fragment_shader(
                &pipeline_data.attributes,
                &uniform_values,
            ),
            vertex_buffer: create_vertex_buffer(
                display,
                &buffers_data.data[..],
                &buffers_data.layout,
            )?,
            uniforms: uniform! {
                time: uniform_values
                    .get("time")
                    .map(|v| match v {
                        UniformValueWrapper::Float(f) => *f,
                        _ => 0.0,
                    })
                    .unwrap_or(0.0),
                color: uniform_values
                    .get("color")
                    .map(|v| match v {
                        UniformValueWrapper::Vec3(v) => *v,
                        _ => [1.0, 1.0, 1.0],
                    })
                    .unwrap_or([1.0, 1.0, 1.0]),
                model: uniform_values
                    .get("model")
                    .map(|v| match v {
                        UniformValueWrapper::Mat4(m) => *m,
                        _ => DEFAULT_MATRIX,
                    })
                    .unwrap_or(DEFAULT_MATRIX),
                view: uniform_values
                    .get("view")
                    .map(|v| match v {
                        UniformValueWrapper::Mat4(m) => *m,
                        _ => DEFAULT_MATRIX,
                    })
                    .unwrap_or(DEFAULT_MATRIX),
                projection: uniform_values
                    .get("projection")
                    .map(|v| match v {
                        UniformValueWrapper::Mat4(m) => *m,
                        _ => DEFAULT_MATRIX,
                    })
                    .unwrap_or(DEFAULT_MATRIX),
                u_light: uniform_values
                    .get("u_light")
                    .map(|v| match v {
                        UniformValueWrapper::Vec3(v) => *v,
                        _ => [1.0, 1.0, 1.0],
                    })
                    .unwrap_or([1.0, 1.0, 1.0]),
            },
            primitive_type: match primitive.as_str() {
                "points" => PrimitiveType::Points,
                "lineStrip" => PrimitiveType::LineStrip,
                "triangleStrip" => PrimitiveType::TriangleStrip,
                "triangles" => PrimitiveType::TrianglesList,
                _ => PrimitiveType::TriangleStrip,
            },
        })
    }

    pub fn build_program(&self, display: &Display<WindowSurface>) -> Result<Program> {
        Program::from_source(display, &self.vertex_shader, &self.fragment_shader, None)
            .map_err(|e| InterpreterError::Custom(e.to_string()))
    }
}
