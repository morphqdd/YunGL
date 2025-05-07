use crate::interpreter::render_statement::pipeline_data::AttributeLayouts;
use crate::interpreter::render_statement::uniform_generator::UniformValueWrapper;
use std::collections::HashMap;

pub struct ShaderGenerator;

impl ShaderGenerator {
    pub fn generate_vertex_shader(
        attributes: &AttributeLayouts,
        uniforms: &HashMap<String, UniformValueWrapper>,
    ) -> String {
        let mut shader = String::from("#version 330 core\n");

        // Генерируем атрибуты
        for (name, attr_type) in &attributes.inputs {
            shader.push_str(&format!("in {} {};\n", attr_type, name));
        }

        for (name, attr_type) in &attributes.outputs {
            shader.push_str(&format!("out {} {};\n", attr_type, name));
        }

        for (name, uniform_type) in uniforms {
            if let UniformValueWrapper::Mat4(_) = uniform_type {
                shader.push_str(&format!("uniform {} {};\n", "mat4", name));
            }
        }

        // Основная функция
        shader.push_str("void main() {\n");
        let position_type = String::from("vec4");
        let position_type = attributes.inputs.get("position").unwrap_or(&position_type);
        let transform = if uniforms.contains_key("model")
            && uniforms.contains_key("view")
            && uniforms.contains_key("projection")
        {
            "projection * view * model * "
        } else {
            ""
        };

        if attributes.inputs.contains_key("color") && attributes.outputs.contains_key("v_color") {
            shader.push_str("    v_color = color;\n");
        }

        if attributes.inputs.contains_key("normal") && attributes.outputs.contains_key("v_normal") {
            shader.push_str("    v_normal = transpose(inverse(mat3(model))) * normal;\n");
        }
        //let transform = String::from("");
        match position_type.as_str() {
            "vec2" => shader.push_str(&format!(
                "    gl_Position = {}vec4(position, 0.0, 1.0);\n",
                transform
            )),
            "vec3" => shader.push_str(&format!(
                "    gl_Position = {}vec4(position, 1.0);\n",
                transform
            )),
            "vec4" => shader.push_str(&format!("    gl_Position = {}{};\n", transform, "position")),
            _ => shader.push_str("    gl_Position = vec4(0.0, 0.0, 0.0, 1.0);\n"),
        }

        shader.push_str("}\n");

        shader
    }

    pub fn generate_fragment_shader(
        attributes: &AttributeLayouts,
        uniforms: &HashMap<String, UniformValueWrapper>,
    ) -> String {
        let mut shader = String::from("#version 330 core\n");
        let mut has_texture = false;
        let mut has_color = false;
        let mut has_light = false;
        for (name, attr_type) in &attributes.outputs {
            shader.push_str(&format!("in {} {};\n", attr_type, name));
        }
        for (name, uniform_type) in uniforms {
            match uniform_type {
                UniformValueWrapper::Texture(_) => {
                    has_texture = true;
                    shader.push_str(&format!("uniform sampler2D {};\n", name));
                }
                UniformValueWrapper::Vec3(_) if name == "color" => {
                    has_color = true;
                    shader.push_str("uniform vec3 color;\n");
                }
                UniformValueWrapper::Vec3(_) if name == "u_light" => {
                    has_light = true;
                    shader.push_str("uniform vec3 u_light;\n");
                }
                _ => {}
            }
        }
        shader.push_str("out vec4 out_color;\n");
        shader.push_str("void main() {\n");
        if has_texture && attributes.inputs.contains_key("uv") {
            if has_color {
                shader.push_str("    out_color = texture(tex, v_uv) * vec4(color, 1.0);\n");
            } else {
                shader.push_str("    out_color = texture(tex, v_uv);\n");
            }
        } else if has_color && has_light {
            shader.push_str("    float brightness = dot(normalize(v_normal), normalize(u_light));\n");
            shader.push_str("    vec3 dark_color = color/2;\n");
            shader.push_str("    vec3 regular_color = color;\n");
            shader.push_str("    out_color = vec4(mix(dark_color, regular_color, brightness), 1.0);\n");
        } else if has_color {
            let mut color = "color".to_string();
            if attributes.outputs.contains_key("v_color") {
                color = color.to_owned() + " * v_color";
            }
            shader.push_str(&format!("    out_color = vec4({color}, 1.0);\n"));
        } else {
            shader.push_str("    out_color = vec4(1.0, 1.0, 1.0, 1.0);\n");
        }
        shader.push_str("}\n");
        shader
    }
}
