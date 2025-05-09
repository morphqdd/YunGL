use crate::interpreter::render_statement::buffers_data::BuffersData;
use crate::interpreter::render_statement::uniform_generator::UniformValueWrapper;
use crate::interpreter::render_statement::vertex::Vertex;
use glium::VertexBuffer;
use glium::index::PrimitiveType;
use std::collections::HashMap;

pub struct Packet {
    pub vertex_shader: String,
    pub fragment_shader: String,
    pub buffer_data: BuffersData,
    pub uniforms: HashMap<String, UniformValueWrapper>,
    pub primitive_type: PrimitiveType,
}
