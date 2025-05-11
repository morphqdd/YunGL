use crate::app::packet::Packet;
use crate::interpreter::Interpreter;
use crate::interpreter::event::InterpreterEvent;
use crate::interpreter::object::Object;
use crate::interpreter::object::callable::Callable;
use crate::interpreter::render_statement::RenderStatement;
use crate::interpreter::render_statement::buffers_data::BuffersData;
use crate::interpreter::render_statement::pipeline_data::{AttributeLayouts, PipelineData};
use crate::interpreter::render_statement::shader_generator::ShaderGenerator;
use crate::interpreter::render_statement::uniform_generator::{
    UniformGenerator, UniformValueWrapper,
};
use crate::interpreter::render_statement::vertex::create_vertex_buffer;
use crate::rc;
use glium::glutin::surface::WindowSurface;
use glium::index::{NoIndices, PrimitiveType};
use glium::uniforms::DynamicUniforms;
use glium::winit::application::ApplicationHandler;
use glium::winit::event::KeyEvent;
use glium::winit::event::{ElementState, StartCause, WindowEvent};
use glium::winit::event_loop::{ActiveEventLoop, EventLoopProxy};
use glium::winit::window::{Window, WindowId};
use glium::{Depth, DepthTest, Display, DrawParameters, Program, Surface};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use notify_debouncer_mini::{DebounceEventResult, DebouncedEvent, Debouncer, new_debouncer};
use rayon::iter::ParallelIterator;
use rayon::prelude::IntoParallelRefIterator;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut, Index};
use std::path::{Path, PathBuf};
use std::process::exit;
use std::sync::atomic::Ordering;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex, RwLock, mpsc};
use std::time::Duration;

pub mod packet;

#[derive(Debug)]
pub enum AppEvent {
    KeyEvent(KeyEvent),
    ReqKeyEvent(String, Callable),
}

pub struct App {
    window: Arc<Window>,
    display: Arc<Display<WindowSurface>>,
    render_statement: Vec<RenderStatement>,
    interpreter: Arc<Mutex<Interpreter>>,
    uniform_generator: Arc<RwLock<UniformGenerator>>,
    path: Arc<PathBuf>,
    watcher: Arc<Debouncer<RecommendedWatcher>>,
    rx_restart: Arc<Mutex<Receiver<()>>>,
    event_channel: (Sender<AppEvent>, Arc<Mutex<Receiver<AppEvent>>>),
    program: Option<Program>,
    vec_vertex: Arc<RwLock<Vec<Object>>>,
    vec_vertex_data: Arc<RwLock<Vec<(Vec<f32>, Vec<String>)>>>,
    vec_attrs: Arc<RwLock<Vec<Object>>>,
    vec_attrs_data: Arc<RwLock<Vec<AttributeLayouts>>>,
}

impl App {
    pub fn new(
        window: Arc<Window>,
        display: Arc<Display<WindowSurface>>,
        proxy: EventLoopProxy<InterpreterEvent>,
        path: PathBuf,
    ) -> Self {
        let path_arc = rc!(path.clone());
        let (tx, rx) = mpsc::channel();
        let mut debouncer = new_debouncer(
            Duration::from_millis(1000),
            move |event: DebounceEventResult| match event {
                Ok(_) => {
                    tx.send(()).unwrap();
                }
                Err(_) => {}
            },
        )
        .unwrap();
        debouncer
            .watcher()
            .watch(&path, RecursiveMode::NonRecursive)
            .unwrap();

        let (tx_event, rx_event) = mpsc::channel();

        Self {
            window,
            display,
            interpreter: rc!(Mutex::new(Interpreter::new(proxy.clone(), path.clone()))),
            render_statement: vec![],
            uniform_generator: rc!(RwLock::new(UniformGenerator::new())),
            path: path_arc,
            watcher: rc!(debouncer),
            rx_restart: rc!(Mutex::new(rx)),
            event_channel: (tx_event, rc!((Mutex::new(rx_event)))),
            program: None,
            vec_vertex: rc!(RwLock::new(Vec::new())),
            vec_vertex_data: rc!(RwLock::new(Vec::<(Vec<f32>, Vec<String>)>::new())),
            vec_attrs: rc!(RwLock::new(Vec::new())),
            vec_attrs_data: rc!(RwLock::new(Vec::<AttributeLayouts>::new())),
        }
    }
}

impl ApplicationHandler<InterpreterEvent> for App {
    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
        match cause {
            StartCause::Init => {
                let interpreter = self.interpreter.clone();
                let must_call_handler = interpreter.lock().unwrap().get_must_call_handler();
                let rx = self.rx_restart.clone();
                let flag = interpreter.lock().unwrap().get_cancel_flag();
                std::thread::spawn(move || {
                    loop {
                        let mut interpreter = interpreter.lock().unwrap();
                        match interpreter.run_script() {
                            Ok(_) => {
                                if interpreter.get_cancel_flag().load(Ordering::Relaxed) {
                                    interpreter.get_cancel_flag().swap(false, Ordering::Relaxed);
                                    continue;
                                }
                                return;
                            }
                            Err(err) => {
                                println!("{}", err);
                                exit(65);
                            }
                        }
                    }
                });

                std::thread::spawn(move || {
                    while rx.lock().unwrap().recv().is_ok() {
                        flag.swap(true, Ordering::Relaxed);
                    }
                });

                let (_, rx_event) = self.event_channel.clone();
                std::thread::spawn(move || {
                    let mut event_map: HashMap<String, Callable> = HashMap::new();
                    while let Ok(event) = rx_event.lock().unwrap().recv() {
                        match event {
                            AppEvent::KeyEvent(key_event)
                                if key_event.state == ElementState::Pressed =>
                            {
                                let Some(key) = key_event.logical_key.to_text() else {
                                    continue;
                                };
                                if let Some(callable) = event_map.get(key).cloned() {
                                    must_call_handler.send(callable).unwrap()
                                }
                            }
                            AppEvent::ReqKeyEvent(key, callable) => {
                                event_map.insert(key, callable);
                            }
                            _ => {}
                        }
                    }
                });
            }
            _ => {}
        }
    }

    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: InterpreterEvent) {
        use Object::{Dictionary, List, Number, String as ObjString};

        const ATTRIBUTES: &str = "attributes";
        const UNIFORM: &str = "uniform";
        const LAYOUT: &str = "layout";
        const DATA: &str = "data";
        const PRIMITIVE: &str = "primitive";
        const VERTEX_SHADER: &str = "vertex";
        const FRAGMENT_SHADER: &str = "fragment";
        const LIGHTS: &str = "lights";

        match event {
            InterpreterEvent::Render(List(list)) => {
                let packets = Arc::new(RwLock::new(Vec::new()));
                let uniform_generator = self.uniform_generator.clone();
                let mut vec_vertex = self.vec_vertex.clone();
                let mut vec_vertex_data = self.vec_vertex_data.clone();
                let mut vec_attrs = self.vec_attrs.clone();
                let mut vec_attrs_data = self.vec_attrs_data.clone();
                list.read().unwrap().par_iter().for_each(|elm| {
                    let Some(pipeline) = elm.get_field(Number(0.0)) else {
                        return;
                    };
                    let Some(vertex) = elm.get_field(Number(1.0)) else {
                        return;
                    };

                    let (data, layout) = if vec_vertex.read().unwrap().contains(&vertex) {
                        let (i, _) = vec_vertex
                            .read()
                            .unwrap()
                            .iter()
                            .enumerate()
                            .find(|(i, x)| x == &&vertex)
                            .unwrap();
                        vec_vertex_data.read().unwrap().get(i).unwrap().clone()
                    } else {
                        let layout_list = match vertex.get_field(ObjString(LAYOUT.into())) {
                            Some(List(layout)) => layout,
                            _ => return,
                        };

                        let layout_list = layout_list.read().unwrap();

                        let mut layout = Vec::with_capacity(layout_list.len());
                        let mut keys = Vec::with_capacity(layout_list.len() * 2);

                        for obj in layout_list.iter() {
                            if let ObjString(s) = obj {
                                layout.push(s.clone());
                                match s.as_str() {
                                    "vec2" => keys.extend(["x", "y"]),
                                    "vec3" => keys.extend(["x", "y", "z"]),
                                    "uv" => keys.extend(["u", "v"]),
                                    "normal" => keys.extend(["nx", "ny", "nz"]),
                                    "color" => keys.extend(["r", "g", "b"]),
                                    _ => {}
                                }
                            }
                        }

                        let data_list = match vertex.get_field(ObjString(DATA.into())) {
                            Some(List(data)) => data,
                            _ => return,
                        };

                        let data_list = data_list.read().unwrap();

                        let mut data = Vec::with_capacity(data_list.len() * keys.len());

                        for obj in data_list.iter() {
                            if let Dictionary(fields) = obj {
                                for &key in &keys {
                                    match fields.read().unwrap().get(key) {
                                        Some(Number(n)) => data.push(*n as f32),
                                        _ => panic!("Expected number for key {}", key),
                                    }
                                }
                            }
                        }

                        vec_vertex.write().unwrap().push(vertex.clone());
                        vec_vertex_data
                            .write()
                            .unwrap()
                            .push((data.clone(), layout.clone()));
                        (data, layout)
                    };

                    let uniform = pipeline
                        .get_field(ObjString(UNIFORM.into()))
                        .unwrap_or(Dictionary(rc!(RwLock::new(HashMap::new()))));

                    let raw_attrs_ = match pipeline.get_field(ObjString(ATTRIBUTES.into())) {
                        Some(obj) => obj,
                        _ => Dictionary(rc!(RwLock::new(HashMap::new()))),
                    };

                    let attrs_layout = if vec_attrs.read().unwrap().contains(&raw_attrs_) {
                        let (i, _) = vec_attrs
                            .read()
                            .unwrap()
                            .iter()
                            .enumerate()
                            .find(|(i, x)| x == &&raw_attrs_)
                            .unwrap();
                        vec_attrs_data.read().unwrap().get(i).unwrap().clone()
                    } else {
                        let raw_attrs = match raw_attrs_.clone() {
                            Dictionary(m) => m.clone(),
                            _ => rc!(RwLock::new(HashMap::new())),
                        };

                        let mut attrs_in = HashMap::new();
                        let mut attrs_out = HashMap::new();

                        if let Some(Dictionary(ins)) = raw_attrs.read().unwrap().get("in") {
                            for (name, typ_obj) in ins.read().unwrap().iter() {
                                if let Object::String(s) = typ_obj {
                                    attrs_in.insert(name.clone(), s.clone());
                                }
                            }
                        }

                        if let Some(Dictionary(outs)) = raw_attrs.read().unwrap().get("out") {
                            for (name, typ_obj) in outs.read().unwrap().iter() {
                                if let Object::String(s) = typ_obj {
                                    attrs_out.insert(name.clone(), s.clone());
                                }
                            }
                        }

                        let attrs = AttributeLayouts {
                            inputs: attrs_in,
                            outputs: attrs_out,
                        };

                        vec_attrs.write().unwrap().push(raw_attrs_);
                        vec_attrs_data.write().unwrap().push(attrs.clone());
                        attrs
                    };

                    let mut uniforms = uniform_generator
                        .write()
                        .unwrap()
                        .generate_uniforms(&uniform)
                        .expect("Failed to generate uniforms");

                    let lights = match pipeline.get_field(ObjString(LIGHTS.into())) {
                        Some(Dictionary(lights)) => lights,
                        _ => rc!(RwLock::new(HashMap::new()))
                    };

                    let mut light_data = Vec::new();

                    for (name, obj) in lights.read().unwrap().iter() {
                        let opt = match obj {
                            Dictionary(l) => l.clone(),
                            _ => panic!("Expected light options"),
                        };
                        let List(list) = opt.read().unwrap().get("position").clone().unwrap().clone() else { panic!("Expected list of positions"); };
                        let mut pos = [0f32;3];
                        for (i, num) in list.read().unwrap().iter().enumerate() {
                            let Number(num) = *num else { panic!("Expected number"); };
                            pos[i] = num as f32;
                        }
                        let position = UniformValueWrapper::Vec3(pos);
                        uniforms.insert(format!("u_light_{}", name), position);

                        let List(list) = opt.read().unwrap().get("color").clone().unwrap().clone() else { panic!("Expected list of positions"); };
                        let mut color = [0f32;3];
                        for (i, num) in list.read().unwrap().iter().enumerate() {
                            let Number(num) = *num else { panic!("Expected number"); };
                            color[i] = num as f32;
                        }
                        let color = UniformValueWrapper::Vec3(color);
                        uniforms.insert(format!("u_light_color_{}", name), color);

                        light_data.push((name.clone(), format!("u_light_{}", name), format!("u_light_color_{}", name)));
                    }

                    let primitive = match pipeline.get_field(ObjString(PRIMITIVE.into())) {
                        Some(Object::String(m)) => m,
                        _ => "triangleStrip".into(),
                    };

                    //println!("DATA: {:?}", attrs_in);

                    let vertex_shader = match pipeline.get_field(ObjString(VERTEX_SHADER.into())) {
                        Some(Object::String(m)) => Some(m),
                        _ => None,
                    };

                    let fragment_shader =
                        match pipeline.get_field(ObjString(FRAGMENT_SHADER.into())) {
                            Some(Object::String(m)) => Some(m),
                            _ => None,
                        };

                    // println!(
                    //     "Vertex: {}\nFragment: {}",
                    //     render_statement.vertex_shader, render_statement.fragment_shader
                    // );

                    packets.write().unwrap().push(Packet {
                        vertex_shader: if let Some(vertex_shader) = vertex_shader {
                            vertex_shader
                        } else {
                            let vert = ShaderGenerator::generate_vertex_shader(&attrs_layout, &uniforms);
                            //println!("Generated vertex shader: {}", vert);
                            vert
                        },
                        fragment_shader: if let Some(fragment_shader) = fragment_shader {
                            fragment_shader
                        } else {
                            let frag = ShaderGenerator::generate_fragment_shader(&attrs_layout, &uniforms, &light_data);
                            //println!("Generated fragment shader: {}", frag);
                            frag
                        },
                        buffer_data: BuffersData { data, layout },
                        uniforms,
                        primitive_type: match primitive.as_str() {
                            "points" => PrimitiveType::Points,
                            "lineStrip" => PrimitiveType::LineStrip,
                            "triangleStrip" => PrimitiveType::TriangleStrip,
                            "triangles" => PrimitiveType::TrianglesList,
                            _ => PrimitiveType::TriangleStrip,
                        },
                    });
                });

                for packet in packets.write().unwrap().iter() {
                    self.render_statement.push(RenderStatement {
                        vertex_shader: packet.vertex_shader.clone(),
                        fragment_shader: packet.fragment_shader.clone(),
                        vertex_buffer: create_vertex_buffer(
                            self.display.as_ref(),
                            &packet.buffer_data.data[..],
                            &packet.buffer_data.layout,
                        )
                        .unwrap(),
                        uniforms: packet.uniforms.clone(),
                        primitive_type: packet.primitive_type.clone(),
                    });
                }
            }
            InterpreterEvent::GetWindowDimensions(tx) => {
                tx.send(self.window.inner_size().into()).unwrap();
            }
            InterpreterEvent::RegKeyEvent(key, handler) => self
                .event_channel
                .0
                .send(AppEvent::ReqKeyEvent(key, handler))
                .unwrap(),
            InterpreterEvent::None => {}
            _ => {}
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                let mut target = self.display.draw();
                target.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);
                for render_statement in &self.render_statement {
                    let mut uniforms = DynamicUniforms::new();

                    for (key, uniform) in &render_statement.uniforms {
                        uniforms.add(
                            &key,
                            match uniform {
                                UniformValueWrapper::Float(num) => num,
                                UniformValueWrapper::Vec3(vec) => vec,
                                UniformValueWrapper::Mat4(mat) => mat,
                            },
                        )
                    }

                    if self.program.is_none() {
                        self.program = Some(render_statement.build_program(&self.display).unwrap());
                    }

                    target
                        .draw(
                            &render_statement.vertex_buffer,
                            &NoIndices(render_statement.primitive_type),
                            self.program.as_ref().unwrap(),
                            &uniforms,
                            &DrawParameters {
                                depth: Depth {
                                    test: DepthTest::IfLessOrEqual,
                                    write: true,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                        )
                        .unwrap();
                }
                self.render_statement.clear();
                target.finish().unwrap();
            }
            WindowEvent::Resized(size) => self.display.resize(size.into()),
            WindowEvent::KeyboardInput { event, .. } => {
                self.event_channel
                    .0
                    .send(AppEvent::KeyEvent(event))
                    .unwrap();
            }
            _ => {}
        }
    }
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if !self.render_statement.is_empty() {
            self.window.request_redraw();
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}
