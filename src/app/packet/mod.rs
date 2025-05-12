use crate::interpreter::render_statement::buffers_data::BuffersData;
use crate::interpreter::render_statement::uniform_generator::UniformValueWrapper;
use glium::index::PrimitiveType;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Packet {
    pub vertex_shader: String,
    pub fragment_shader: String,
    pub buffer_data: BuffersData,
    pub uniforms: HashMap<String, UniformValueWrapper>,
    pub primitive_type: PrimitiveType,
    pub light_names: Vec<String>,
}
