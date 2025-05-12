use crate::interpreter::error::{InterpreterError, Result};
use crate::interpreter::object::Object;
use crate::rc;
use glium::glutin::surface::WindowSurface;
use glium::texture::RawImage2d;
use glium::{Display, Texture2d};
use image::{DynamicImage, RgbaImage};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Clone, Debug)]
pub enum UniformValueWrapper {
    Float(f32),
    Vec3([f32; 3]),
    Mat4([[f32; 4]; 4]),
    Sampler2D(DynamicImage),
}

pub struct UniformGenerator {
    default: Object,
}

impl UniformGenerator {
    pub(crate) fn new() -> Self {
        Self {
            default: Self::default_common_uniforms(),
        }
    }

    fn default_common_uniforms() -> Object {
        let identity_matrix = vec![
            Object::List(rc!(RwLock::new(vec![
                Object::Number(1.0),
                Object::Number(0.0),
                Object::Number(0.0),
                Object::Number(0.0),
            ]))),
            Object::List(rc!(RwLock::new(vec![
                Object::Number(0.0),
                Object::Number(1.0),
                Object::Number(0.0),
                Object::Number(0.0),
            ]))),
            Object::List(rc!(RwLock::new(vec![
                Object::Number(0.0),
                Object::Number(0.0),
                Object::Number(1.0),
                Object::Number(0.0),
            ]))),
            Object::List(rc!(RwLock::new(vec![
                Object::Number(0.0),
                Object::Number(0.0),
                Object::Number(0.0),
                Object::Number(1.0),
            ]))),
        ];

        let matrix = Object::Dictionary(Arc::new(RwLock::new(HashMap::from([
            ("type".to_string(), Object::String("mat4".to_string())),
            (
                "value".to_string(),
                Object::List(rc!(RwLock::new(identity_matrix.clone()))),
            ),
        ]))));
        Object::Dictionary(Arc::new(RwLock::new(HashMap::from([
            (
                "time".to_string(),
                Object::Dictionary(Arc::new(RwLock::new(HashMap::from([
                    ("type".to_string(), Object::String("float".to_string())),
                    ("value".to_string(), Object::Number(0.0)),
                ])))),
            ),
            (
                "color".to_string(),
                Object::Dictionary(Arc::new(RwLock::new(HashMap::from([
                    ("type".to_string(), Object::String("vec3".to_string())),
                    (
                        "value".to_string(),
                        Object::List(rc!(RwLock::new(vec![
                            Object::Number(1.0),
                            Object::Number(1.0),
                            Object::Number(1.0),
                        ]))),
                    ),
                ])))),
            ),
            ("model".to_string(), matrix.clone()),
            ("view".to_string(), matrix.clone()),
            ("projection".to_string(), matrix.clone()),
            (
                "light_position".to_string(),
                Object::Dictionary(Arc::new(RwLock::new(HashMap::from([
                    ("type".to_string(), Object::String("vec3".to_string())),
                    (
                        "value".to_string(),
                        Object::List(rc!(RwLock::new(vec![
                            Object::Number(0.0),
                            Object::Number(0.0),
                            Object::Number(5.0),
                        ]))),
                    ),
                ])))),
            ),
        ]))))
    }
    pub fn generate_uniforms(
        &mut self,
        uniforms: &Object,
    ) -> Result<HashMap<String, UniformValueWrapper>> {
        let user_uniforms = match uniforms {
            Object::Dictionary(dict) => dict,
            _ => {
                return Err(InterpreterError::Custom(
                    "Uniforms must be a dictionary".into(),
                ));
            }
        }
        .read()
        .unwrap();
        let Object::Dictionary(common_uniforms) = self.default.clone() else {
            panic!("Expected dictionary")
        };

        let mut uniform_values = HashMap::new();
        let mut merged_uniforms = common_uniforms.write().unwrap();
        merged_uniforms.extend(user_uniforms.iter().map(|(k, v)| (k.clone(), v.clone())));

        for (name, uniform) in merged_uniforms.iter() {
            let uniform_dict = match uniform {
                Object::Dictionary(dict) => dict,
                _ => {
                    return Err(InterpreterError::Custom(
                        "Uniform must be a dictionary".into(),
                    ));
                }
            }
            .read()
            .unwrap();

            let uniform_type = match uniform_dict.get("type") {
                Some(Object::String(s)) => s,
                _ => {
                    return Err(InterpreterError::Custom(
                        "Uniform type must be a string".into(),
                    ));
                }
            };

            let value = match uniform_dict.get("value") {
                Some(v) => v,
                _ => return Err(InterpreterError::Custom("Uniform value required".into())),
            };

            match uniform_type.as_str() {
                "float" => {
                    let float_value = match value {
                        Object::Number(n) => *n as f32,
                        _ => {
                            return Err(InterpreterError::Custom(
                                "Float value must be a number".into(),
                            ));
                        }
                    };
                    uniform_values.insert(name.clone(), UniformValueWrapper::Float(float_value));
                }
                "vec3" => {
                    let vec3_value = match value {
                        Object::List(list) if list.read().unwrap().len() == 3 => {
                            let mut arr = [0.0f32; 3];
                            for (i, item) in list.read().unwrap().iter().enumerate() {
                                arr[i] = match item {
                                    Object::Number(n) => *n as f32,
                                    _ => {
                                        return Err(InterpreterError::Custom(
                                            "Vec3 value must be numbers".into(),
                                        ));
                                    }
                                };
                            }
                            arr
                        }
                        _ => {
                            return Err(InterpreterError::Custom(
                                "Vec3 value must be a list of 3 numbers".into(),
                            ));
                        }
                    };
                    uniform_values.insert(name.clone(), UniformValueWrapper::Vec3(vec3_value));
                }
                "mat4" => {
                    let mat4_value = match value {
                        Object::List(list) => {
                            //println!("list: {:?}", list);
                            let mut arr = [[0.0f32; 4]; 4];
                            for (i, item) in list.read().unwrap().iter().enumerate() {
                                arr[i] = match item {
                                    Object::List(n) => {
                                        let mut buffer = [0.0f32; 4];
                                        for (i, obj) in n.read().unwrap().iter().enumerate() {
                                            buffer[i] = if let Object::Number(n) = obj {
                                                *n as f32
                                            } else {
                                                0.0f32
                                            };
                                        }
                                        //println!("{:?}", buffer);
                                        buffer
                                    }
                                    _ => {
                                        return Err(InterpreterError::Custom(
                                            "Mat4 value must be list".into(),
                                        ));
                                    }
                                };
                            }
                            arr
                        }
                        _ => {
                            return Err(InterpreterError::Custom(
                                "Mat4 value must be a matrix 4x4".into(),
                            ));
                        }
                    };

                    //println!("mat4: {:?}", mat4_value);

                    uniform_values.insert(name.clone(), UniformValueWrapper::Mat4(mat4_value));
                }
                "sampler2D" => {
                    let Object::NativeObject(native) = value.clone() else {
                        panic!("Expected Native object")
                    };
                    if let Ok(image) = native.extract().downcast::<DynamicImage>() {
                        uniform_values.insert(name.clone(), UniformValueWrapper::Sampler2D(*image));
                    }
                }
                _ => {
                    return Err(InterpreterError::Custom(format!(
                        "Unsupported uniform type: {}",
                        uniform_type
                    )));
                }
            }
        }

        //println!("----UNIFORMS: {:?}", uniform_values);

        Ok(uniform_values)
    }
}
pub fn load_texture(display: &Display<WindowSurface>, image: RgbaImage) -> Result<Texture2d> {
    let image_dimensions = image.dimensions();
    let image = RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);

    let texture = Texture2d::new(display, image).unwrap();
    Ok(texture)
}
