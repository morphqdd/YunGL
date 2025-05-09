use crate::interpreter::render_statement::uniform_generator::UniformValueWrapper;
use std::collections::HashMap;

pub struct PipelineData {
    pub attributes: AttributeLayouts,
    pub uniforms: HashMap<String, UniformValueWrapper>,
}

#[derive(Debug, Clone)]
pub struct AttributeLayouts {
    pub inputs: HashMap<String, String>,
    pub outputs: HashMap<String, String>,
}
