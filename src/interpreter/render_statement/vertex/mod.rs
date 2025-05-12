use crate::interpreter::error::Result;
use glium::glutin::surface::WindowSurface;
use glium::{Display, VertexBuffer, implement_vertex};

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    position: [f32; 4], // Максимум vec4
    uv: [f32; 2],       // Текстурные координаты
    normal: [f32; 3],   // Нормали
    color: [f32; 3],
}
implement_vertex!(Vertex, position, uv, normal, color);

pub fn create_vertex_buffer(
    display: &Display<WindowSurface>,
    data: &[f32],
    layout: &Vec<String>,
) -> Result<VertexBuffer<Vertex>> {
    let mut position_size = 4; // По умолчанию vec4
    let mut has_uv = false;
    let mut has_normal = false;
    let mut has_color = false;

    for comp in layout {
        match comp.as_str() {
            "vec2" => position_size = 2,
            "vec3" => position_size = 3,
            "vec4" => position_size = 4,
            "uv" => has_uv = true,
            "normal" => has_normal = true,
            "color" => has_color = true,
            _ => return Err(format!("Unsupported layout component: {}", comp).into()),
        }
    }
    //println!("position_size: {}", position_size);
    let mut vertex_data = Vec::new();
    let stride = position_size
        + (if has_uv { 2 } else { 0 })
        + (if has_normal { 3 } else { 0 })
        + (if has_color { 3 } else { 0 });
    for chunk in data.chunks(stride) {
        let mut position = [0.0; 4];
        position[3] = 1.0;

        let mut uv = [0.0; 2];
        let mut normal = [0.0; 3];
        let mut color = [1.0; 3];
        let mut offset = 0;

        // Позиция
        for i in 0..position_size {
            position[i] = chunk.get(offset).copied().unwrap_or(0.0);
            offset += 1;
        }

        // Нормали
        if has_normal {
            for i in 0..3 {
                normal[i] = chunk.get(offset).copied().unwrap_or(0.0);
                offset += 1;
            }
        }

        if has_uv {
            for i in 0..2 {
                uv[i] = chunk.get(offset).copied().unwrap_or(0.0);
                offset += 1;
            }
        }

        if has_color {
            for i in 0..3 {
                color[i] = chunk.get(offset).copied().unwrap_or(1.0);
                offset += 1;
            }
        }

        vertex_data.push(Vertex {
            position,
            uv,
            normal,
            color,
        });
        //println!("VERTEX: {:?}", vertex_data);
    }
    //println!("{:?}", vertex_data);

    VertexBuffer::new(display, &vertex_data).map_err(|e| e.to_string().into())
}
