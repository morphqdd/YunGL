use crate::interpreter::error::{InterpreterError, Result};
use crate::interpreter::render_statement::buffers_data::BuffersData;
use crate::interpreter::render_statement::pipeline_data::PipelineData;
use crate::interpreter::render_statement::shader_generator::ShaderGenerator;
use crate::interpreter::render_statement::uniform_generator::{UniformValueWrapper, load_texture};
use crate::interpreter::render_statement::vertex::{Vertex, create_vertex_buffer};
use glium::glutin::surface::WindowSurface;
use glium::texture::SrgbTexture2d;
use glium::uniforms::{AsUniformValue, EmptyUniforms, Uniforms, UniformsStorage};
use glium::{Display, Program, Texture2d, VertexBuffer, uniform};
use image::{ImageFormat, load};
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Arc;

pub mod buffers_data;
pub mod pipeline_data;
pub mod shader_generator;
pub mod uniform_generator;
pub mod vertex;

pub type UniformType<'n> = UniformsStorage<
    'n,
    [f32; 3],
    UniformsStorage<
        'n,
        &'n Texture2d,
        UniformsStorage<
            'n,
            [[f32; 4]; 4],
            UniformsStorage<
                'n,
                [[f32; 4]; 4],
                UniformsStorage<
                    'n,
                    [[f32; 4]; 4],
                    UniformsStorage<'n, [f32; 3], UniformsStorage<'n, f32, EmptyUniforms>>,
                >,
            >,
        >,
    >,
>;

pub struct RenderStatement {
    pub vertex_shader: String,
    pub fragment_shader: String,
    pub vertex_buffer: VertexBuffer<Vertex>,
    pub uniforms: UniformType<'static>,
}

impl RenderStatement {
    pub fn new(
        display: &Display<WindowSurface>,
        pipeline_data: PipelineData,
        buffers_data: BuffersData,
    ) -> Result<Self> {
        let uniform_values = pipeline_data.uniforms;
        let time = uniform_values
            .get("time")
            .map(|v| match v {
                UniformValueWrapper::Float(f) => *f,
                _ => 0.0,
            })
            .unwrap_or(0.0);
        let color = uniform_values
            .get("color")
            .map(|v| match v {
                UniformValueWrapper::Vec3(v) => *v,
                _ => [1.0, 1.0, 1.0],
            })
            .unwrap_or([1.0, 1.0, 1.0]);
        let model = uniform_values
            .get("model")
            .map(|v| match v {
                UniformValueWrapper::Mat4(m) => *m,
                _ => [[1.0, 0.0, 0.0, 0.0]; 4],
            })
            .unwrap_or([
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ]);
        let view = uniform_values
            .get("view")
            .map(|v| match v {
                UniformValueWrapper::Mat4(m) => *m,
                _ => [[1.0, 0.0, 0.0, 0.0]; 4],
            })
            .unwrap_or([
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ]);
        let projection = uniform_values
            .get("projection")
            .map(|v| match v {
                UniformValueWrapper::Mat4(m) => *m,
                _ => [[1.0, 0.0, 0.0, 0.0]; 4],
            })
            .unwrap_or([
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ]);
        let texture = uniform_values
            .get("texture")
            .map(|v| match v {
                UniformValueWrapper::Texture(t) => t.clone(),
                _ => Box::leak(Box::new(
                    load_texture(
                        display,
                        load(
                            Cursor::new(include_bytes!("../../../assets/skate_board_kat.jpg")),
                            ImageFormat::Jpeg,
                        )
                        .unwrap()
                        .to_rgba8(),
                    )
                    .unwrap(),
                )),
            })
            .unwrap_or(Box::leak(Box::new(load_texture(
                display,
                load(
                    Cursor::new(include_bytes!("../../../assets/skate_board_kat.jpg")),
                    ImageFormat::Jpeg,
                )
                .unwrap()
                .to_rgba8(),
            )?)));
        let light_position = uniform_values
            .get("light_position")
            .map(|v| match v {
                UniformValueWrapper::Vec3(v) => *v,
                _ => [0.0, 0.0, 5.0],
            })
            .unwrap_or([0.0, 0.0, 5.0]);

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
            uniforms: uniform! {
                time: time,
                color: color,
                model: model,
                view: view,
                projection: projection,
                texture: texture,
                light_position: light_position
            },
        })
    }

    pub fn build_program(&self, display: &Display<WindowSurface>) -> Result<Program> {
        Program::from_source(display, &self.vertex_shader, &self.fragment_shader, None)
            .map_err(|e| InterpreterError::Custom(e.to_string()))
    }
}
