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

        if uniforms.contains_key("model") && uniforms.contains_key("view") {
            shader.push_str("mat4 modelview = view * model;\n")
        } else {
            shader.push_str("mat4 modelview = model;\n")
        };
        if attributes.inputs.contains_key("normal") && attributes.outputs.contains_key("v_normal") {
            shader.push_str("    v_normal = transpose(inverse(mat3(view * model))) * normal;\n");
        }

        if attributes.inputs.contains_key("color") && attributes.outputs.contains_key("v_color") {
            shader.push_str("    v_color = color;\n");
        }

        let transform = "projection * view * model * ";
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

        if attributes.outputs.contains_key("v_position") {
            shader.push_str("    v_position = vec3(view * model * position);\n");
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
        let mut has_view_pos = false;
        let mut has_specular = false;

        // Входящие данные из вершинного шейдера
        for (name, attr_type) in &attributes.outputs {
            shader.push_str(&format!("in {} {};\n", attr_type, name));
        }

        // Юниформы
        for (name, uniform_type) in uniforms {
            match (name.as_str(), uniform_type) {
                ("color", UniformValueWrapper::Vec3(_)) => {
                    has_color = true;
                    shader.push_str("uniform vec3 color;\n");
                }
                ("u_light", UniformValueWrapper::Vec3(_)) => {
                    has_light = true;
                    shader.push_str("uniform vec3 u_light;\n");
                }
                ("u_light_color", UniformValueWrapper::Vec3(_)) => {
                    shader.push_str("uniform vec3 u_light_color;\n");
                }
                ("u_view_pos", UniformValueWrapper::Vec3(_)) => {
                    has_view_pos = true;
                    shader.push_str("uniform vec3 u_view_pos;\n");
                }
                ("specular_strength", UniformValueWrapper::Float(_)) => {
                    has_specular = true;
                    shader.push_str("uniform float specular_strength;\n");
                }
                ("shininess", UniformValueWrapper::Float(_)) => {
                    has_specular = true;
                    shader.push_str("uniform float shininess;\n");
                }
                _ => {}
            }
        }

        shader.push_str("out vec4 out_color;\n");
        shader.push_str("void main() {\n");

        if has_light
            && attributes.outputs.contains_key("v_normal")
            && attributes.outputs.contains_key("v_position")
        {
            shader.push_str("    vec3 norm = normalize(v_normal);\n");
            shader.push_str("    vec3 light_dir = normalize(u_light - v_position);\n");

            // ambient
            shader.push_str("    vec3 ambient = 0.1 * u_light_color;\n");

            // diffuse
            shader.push_str("    float diff = max(dot(norm, light_dir), 0.0);\n");
            shader.push_str("    vec3 diffuse = diff * u_light_color;\n");

            // specular
            if has_view_pos && has_specular {
                shader.push_str("    vec3 view_dir = normalize(u_view_pos - v_position);\n");
                shader.push_str("    vec3 halfway_dir = normalize(light_dir + view_dir);\n");
                shader.push_str(
                    "    float spec = pow(max(dot(norm, halfway_dir), 0.0), shininess);\n",
                );
                shader.push_str("    vec3 specular = specular_strength * spec * u_light_color;\n");
            } else {
                shader.push_str("    vec3 specular = vec3(0.0);\n");
            }

            // базовый цвет
            if has_texture && attributes.outputs.contains_key("v_uv") {
                shader.push_str("    vec3 base_color = texture(tex, v_uv).rgb;\n");
            } else if has_color {
                shader.push_str("    vec3 base_color = color;\n");
            } else {
                shader.push_str("    vec3 base_color = vec3(1.0);\n");
            }

            // результат
            shader.push_str("    vec3 lighting = (ambient + diffuse + specular) * base_color;\n");
            shader.push_str("    out_color = vec4(lighting, 1.0);\n");
        } else if has_color {
            shader.push_str("    out_color = vec4(color, 1.0);\n");
        } else {
            shader.push_str("    out_color = vec4(1.0);\n");
        }

        shader.push_str("}\n");
        shader
    }
}
