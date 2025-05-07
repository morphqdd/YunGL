use crate::interpreter::Interpreter;
use crate::interpreter::event::InterpreterEvent;
use crate::interpreter::object::Object;
use crate::interpreter::object::Object::String;
use crate::interpreter::render_statement::RenderStatement;
use crate::interpreter::render_statement::buffers_data::BuffersData;
use crate::interpreter::render_statement::pipeline_data::{AttributeLayouts, PipelineData};
use crate::interpreter::render_statement::uniform_generator::UniformGenerator;
use crate::rc;
use glium::glutin::surface::WindowSurface;
use glium::index::{NoIndices, PrimitiveType};
use glium::winit::application::ApplicationHandler;
use glium::winit::event::{StartCause, WindowEvent};
use glium::winit::event_loop::{ActiveEventLoop, EventLoopProxy};
use glium::winit::window::{Window, WindowId};
use glium::{Depth, DepthTest, Display, DrawParameters, Surface};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use notify_debouncer_mini::{DebounceEventResult, DebouncedEvent, Debouncer, new_debouncer};
use std::collections::HashMap;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::sync::atomic::Ordering;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex, RwLock, mpsc};
use std::time::Duration;

pub struct App {
    window: Arc<Window>,
    display: Arc<Display<WindowSurface>>,
    render_statement: Option<RenderStatement>,
    interpreter: Arc<Mutex<Interpreter>>,
    uniform_generator: Arc<RwLock<UniformGenerator>>,
    path: Arc<PathBuf>,
    watcher: Arc<Debouncer<RecommendedWatcher>>,
    rx: Arc<Mutex<Receiver<()>>>,
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
            Duration::from_millis(100),
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
        Self {
            window,
            display,
            interpreter: rc!(Mutex::new(Interpreter::new(proxy.clone(), path.clone()))),
            render_statement: None,
            uniform_generator: rc!(RwLock::new(UniformGenerator::new())),
            path: path_arc,
            watcher: rc!(debouncer),
            rx: rc!(Mutex::new(rx)),
        }
    }
}

impl ApplicationHandler<InterpreterEvent> for App {
    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
        match cause {
            StartCause::Init => {
                let interpreter = self.interpreter.clone();
                let rx = self.rx.clone();
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

        if let InterpreterEvent::Render(List(list)) = event {
            for elm in list.read().unwrap().iter() {
                let Some(pipeline) = elm.get_field(Number(0.0)) else {
                    continue;
                };
                let Some(vertex) = elm.get_field(Number(1.0)) else {
                    continue;
                };

                let uniform = pipeline
                    .get_field(ObjString(UNIFORM.into()))
                    .unwrap_or(Dictionary(rc!(RwLock::new(HashMap::new()))));

                let layout_list = match vertex.get_field(ObjString(LAYOUT.into())) {
                    Some(List(layout)) => layout,
                    _ => continue,
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
                    _ => continue,
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

                let raw_attrs = match pipeline.get_field(ObjString(ATTRIBUTES.into())) {
                    Some(Dictionary(m)) => m,
                    _ => rc!(RwLock::new(HashMap::new())),
                };

                // 2. Соберём inputs и outputs
                let mut attrs_in = HashMap::new();
                let mut attrs_out = HashMap::new();

                // Входные
                if let Some(Dictionary(ins)) = raw_attrs.read().unwrap().get("in") {
                    for (name, typ_obj) in ins.read().unwrap().iter() {
                        if let String(s) = typ_obj {
                            attrs_in.insert(name.clone(), s.clone());
                        }
                    }
                }
                // Выходные
                if let Some(Dictionary(outs)) = raw_attrs.read().unwrap().get("out") {
                    for (name, typ_obj) in outs.read().unwrap().iter() {
                        if let String(s) = typ_obj {
                            attrs_out.insert(name.clone(), s.clone());
                        }
                    }
                }

                let uniforms = self
                    .uniform_generator
                    .write()
                    .unwrap()
                    .generate_uniforms(&uniform)
                    .expect("Failed to generate uniforms");

                let primitive = match pipeline.get_field(ObjString(PRIMITIVE.into())) {
                    Some(String(m)) => m,
                    _ => "triangleStrip".into(),
                };

                //println!("DATA: {:?}", attrs_in);

                let render_statement = RenderStatement::new(
                    &self.display,
                    PipelineData {
                        attributes: AttributeLayouts {
                            inputs: attrs_in,
                            outputs: attrs_out,
                        },
                        uniforms,
                    },
                    BuffersData { data, layout },
                    primitive,
                )
                .expect("Failed to create render statement");

                // println!(
                //     "Vertex: {}\nFragment: {}",
                //     render_statement.vertex_shader, render_statement.fragment_shader
                // );

                self.render_statement = Some(render_statement);
            }
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
                if let Some(render_statement) = &self.render_statement {
                    let program = render_statement.build_program(&self.display).unwrap();
                    let mut target = self.display.draw();
                    target.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);
                    target
                        .draw(
                            &render_statement.vertex_buffer,
                            &NoIndices(render_statement.primitive_type),
                            &program,
                            &render_statement.uniforms,
                            &DrawParameters {
                                depth: Depth {
                                    test: DepthTest::IfLessOrEqual,
                                    write: true,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            //&Default::default(),
                        )
                        .unwrap();
                    target.finish().unwrap();
                }
            }
            WindowEvent::Resized(size) => self.display.resize(size.into()),
            _ => {}
        }
    }
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if self.render_statement.is_some() {
            self.window.request_redraw();
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
}
