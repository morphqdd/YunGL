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

        if uniforms.contains_key("model") && attributes.inputs.contains_key("position") {
            shader.push_str("   vec4 world_pos = model * position;\n");
            shader.push_str("   v_world_pos = world_pos.xyz;\n");
        }

        if attributes.inputs.contains_key("normal")
            && attributes.outputs.contains_key("v_world_normal")
        {
            shader.push_str("   v_world_normal = transpose(inverse(mat3(model))) * normal;\n");
        }

        if attributes.inputs.contains_key("color") && attributes.outputs.contains_key("v_color") {
            shader.push_str("   v_color = color;\n");
        }

        let mut transform = String::from("");
        if uniforms.contains_key("projection") {
            transform += "projection * ";
        }
        if uniforms.contains_key("view") {
            transform += "view * ";
        }
        if uniforms.contains_key("model") && attributes.inputs.contains_key("position") {
            transform += "world_pos";
        } else if attributes.inputs.contains_key("position") {
            transform += "position";
        }
        match position_type.as_str() {
            "vec2" => shader.push_str(&format!(
                "    gl_Position = {}vec4(position, 0.0, 1.0);\n",
                transform
            )),
            "vec3" => shader.push_str(&format!(
                "    gl_Position = {}vec4(position, 1.0);\n",
                transform
            )),
            "vec4" => shader.push_str(&format!("    gl_Position = {};\n", transform)),
            _ => shader.push_str("    gl_Position = vec4(0.0, 0.0, 0.0, 1.0);\n"),
        }

        shader.push_str("}\n");

        shader
    }

    pub fn generate_fragment_shader(
        attributes: &AttributeLayouts,
        uniforms: &HashMap<String, UniformValueWrapper>,
        light_data: &Vec<(String, String, String)>,
    ) -> String {
        let mut shader = String::from("#version 330 core\n");

        for (name, attr_type) in &attributes.outputs {
            shader.push_str(&format!("in {} {};\n", attr_type, name));
        }

        for (name, uniform) in uniforms {
            match uniform {
                UniformValueWrapper::Float(_) => {
                    shader.push_str(&format!("uniform float {};\n", name))
                }
                UniformValueWrapper::Vec3(_) => {
                    shader.push_str(&format!("uniform vec3 {};\n", name))
                }
                UniformValueWrapper::Mat4(_) => {
                    shader.push_str(&format!("uniform mat4 {};\n", name))
                }
                UniformValueWrapper::Sampler2D(_) => {
                    shader.push_str(&format!("uniform sampler2D {};\n", name))
                }
            }
        }

        shader.push_str("out vec4 out_color;\n");
        shader.push_str("void main() {\n");

        let has_light_pos = uniforms.keys().any(|name| name.contains("u_light"));
        let has_light_color = uniforms.keys().any(|name| name.contains("u_light_color"));
        let has_specular_strength = uniforms
            .keys()
            .any(|name| name.contains("specular_strength"));
        let has_shininess = uniforms.keys().any(|name| name.contains("shininess"));
        let has_view_pos = uniforms.keys().any(|name| name.contains("u_view_pos"));
        let has_color = uniforms.keys().any(|name| name.as_str() == "color");

        let mut final_color = String::from("    vec3 final_color = ");
        let mut f_color = String::new();
        if has_view_pos
            && has_shininess
            && has_specular_strength
            && has_light_pos
            && has_light_color
            && has_color
            && attributes.outputs.contains_key("v_world_normal")
            && attributes.outputs.contains_key("v_world_pos")
        {
            shader.push_str("   vec3 N = normalize(v_world_normal);\n");
            shader.push_str("   vec3 V = normalize(u_view_pos - v_world_pos);\n");
            shader.push_str("   float rim_factor = 1.0 - max(dot(N,V), 0.0);\n");

            for (i, (name, pos, color)) in light_data.iter().enumerate() {
                shader.push_str(&format!(
                    "  vec3 L_{name} = normalize({pos} - v_world_pos);\n"
                ));
                shader.push_str(&format!("  vec3 H_{name} = normalize(L_{name} + V);\n"));
                shader.push_str(&format!(
                    "  vec3 ambient_{name} = 0.05 * color + 0.05 * {color};\n"
                ));
                shader.push_str(&format!(
                    "  float diff_{name} = max(dot(N, L_{name}), 0.0);\n"
                ));
                shader.push_str(&format!("  vec3 diffuse_{name} = diff_{name} * {color};\n"));
                shader.push_str(&format!(
                    "  float spec_{name} = pow(max(dot(N, H_{name}), 0.0), shininess);\n"
                ));
                shader.push_str(&format!(
                    "  vec3 specular_{name} = specular_strength * spec_{name} * {color};\n"
                ));
                shader.push_str(&format!(
                    "  vec3 rim_{name} = 0.2 * pow(rim_factor, 2.0) * {color};\n"
                ));
                f_color +=
                    &format!("(ambient_{name} + diffuse_{name} + specular_{name} + rim_{name})");
                if light_data.len() - 1 != i {
                    f_color += "+"
                }
            }
        }
        if !f_color.is_empty() {
            final_color += &format!("({f_color}) * color")
        } else {
            final_color += "color";
        }
        shader.push_str(&format!("{final_color};\n"));
        shader.push_str("   final_color = pow(final_color, vec3(1.0 / 2.2));\n");
        shader.push_str("   out_color = vec4(final_color, 1.0);\n");

        shader.push_str("}\n");
        shader
    }
}
