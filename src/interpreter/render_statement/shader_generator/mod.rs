use std::collections::HashMap;

pub struct ShaderGenerator;

impl ShaderGenerator {
    pub fn generate_vertex_shader(
        attributes: &HashMap<String, String>,
        uniforms: &HashMap<String, (String, String)>,
    ) -> String {
        let mut shader = String::from("#version 330 core\n");

        // Генерируем атрибуты
        for (name, attr_type) in attributes {
            shader.push_str(&format!("in {} {};\n", attr_type, name));
        }

        // Генерируем uniforms для матриц
        for (name, (uniform_type, _)) in uniforms {
            if uniform_type == "mat4" {
                shader.push_str(&format!("uniform {} {};\n", uniform_type, name));
            }
        }

        // Основная функция
        shader.push_str("void main() {\n");
        let position_type = String::from("vec4");
        //let position_type = attributes.get("position").unwrap_or(&position_type);
        let transform = if uniforms.contains_key("model")
            && uniforms.contains_key("view")
            && uniforms.contains_key("projection")
        {
            "projection * view * model * "
        } else {
            ""
        };
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
        if attributes.contains_key("normal") {
            shader.push_str("    v_normal = normal;\n");
        }
        if attributes.contains_key("uv") {
            shader.push_str("    v_uv = uv;\n");
        }
        shader.push_str("}\n");

        // Передача данных во фрагментный шейдер
        if attributes.contains_key("normal") {
            shader.push_str("out vec3 v_normal;\n");
        }
        if attributes.contains_key("uv") {
            shader.push_str("out vec2 v_uv;\n");
        }

        shader
    }

    pub fn generate_fragment_shader(
        attributes: &HashMap<String, String>,
        uniforms: &HashMap<String, (String, String)>,
    ) -> String {
        let mut shader = String::from("#version 330 core\n");
        shader.push_str("out vec4 color;\n");

        // Входные данные из вершинного шейдера
        if attributes.contains_key("normal") {
            shader.push_str("in vec3 v_normal;\n");
        }
        if attributes.contains_key("uv") {
            shader.push_str("in vec2 v_uv;\n");
        }

        // Uniforms для текстуры и света
        for (name, (uniform_type, _)) in uniforms {
            if uniform_type == "sampler2D" || uniform_type == "vec3" {
                shader.push_str(&format!("uniform {} {};\n", uniform_type, name));
            }
        }

        // Основная функция
        shader.push_str("void main() {\n");
        if attributes.contains_key("uv") && uniforms.contains_key("texture") {
            shader.push_str("    vec4 tex_color = texture(texture, v_uv);\n");
        } else {
            shader.push_str("    vec4 tex_color = vec4(0.0, 1.0, 0.0, 1.0);\n"); // Зелёный по умолчанию
        }

        if attributes.contains_key("normal") && uniforms.contains_key("light_position") {
            shader.push_str(
                "    vec3 norm = normalize(v_normal);\n\
                 vec3 light_dir = normalize(light_position - vec3(gl_FragCoord.xyz));\n\
                 float diff = max(dot(norm, light_dir), 0.0);\n\
                 vec3 diffuse = diff * vec3(1.0, 1.0, 1.0);\n\
                 color = tex_color * vec4(diffuse, 1.0);\n",
            );
        } else {
            shader.push_str("    color = tex_color;\n");
        }
        shader.push_str("}\n");

        shader
    }
}
