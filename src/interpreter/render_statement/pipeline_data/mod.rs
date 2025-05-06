use crate::interpreter::render_statement::uniform_generator::UniformValueWrapper;
use glium::uniforms::UniformValue;
use std::collections::HashMap;

pub struct PipelineData {
    pub attributes: HashMap<String, String>,
    pub uniforms: HashMap<String, UniformValueWrapper>,
}
