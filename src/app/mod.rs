use crate::interpreter::Interpreter;
use crate::interpreter::environment::Environment;
use crate::interpreter::event::InterpreterEvent;
use crate::interpreter::object::Object;
use crate::interpreter::render_statement::RenderStatement;
use crate::interpreter::render_statement::buffers_data::BuffersData;
use crate::interpreter::render_statement::pipeline_data::PipelineData;
use crate::interpreter::render_statement::uniform_generator::UniformGenerator;
use glium::glutin::surface::WindowSurface;
use glium::index::{NoIndices, PrimitiveType};
use glium::winit::application::ApplicationHandler;
use glium::winit::event::{StartCause, WindowEvent};
use glium::winit::event_loop::{ActiveEventLoop, EventLoopProxy};
use glium::winit::window::{Window, WindowId};
use glium::{Display, Surface};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::exit;
use std::sync::{Arc, Mutex, RwLock};

pub struct App {
    window: Arc<Window>,
    display: Arc<Display<WindowSurface>>,
    proxy: EventLoopProxy<InterpreterEvent>,
    render_statement: Option<RenderStatement>,
    interpreter: Arc<Mutex<Interpreter>>,
    uniform_generator: Arc<RwLock<UniformGenerator>>,
}

impl App {
    pub fn new(
        window: Arc<Window>,
        display: Arc<Display<WindowSurface>>,
        proxy: EventLoopProxy<InterpreterEvent>,
        path_buf: PathBuf,
    ) -> Self {
        Self {
            window,
            display,
            interpreter: Arc::new(Mutex::new(Interpreter::new(proxy.clone(), path_buf))),
            proxy,
            render_statement: None,
            uniform_generator: Arc::new(RwLock::new(UniformGenerator::new())),
        }
    }
}

impl ApplicationHandler<InterpreterEvent> for App {
    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
        match cause {
            StartCause::Init => {
                let mut interpreter = self.interpreter.clone();
                std::thread::spawn(move || {
                    if let Err(err) = interpreter.lock().unwrap().run_script() {
                        println!("{}", err);
                        exit(65);
                    }
                });
                //println!("Init ended")
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

        if let InterpreterEvent::Render(List(list)) = event {
            for elm in list {
                let Some(pipeline) = elm.get_field(Number(0.0)) else {
                    continue;
                };
                let Some(vertex) = elm.get_field(Number(1.0)) else {
                    continue;
                };

                // Fast access to attributes
                let attributes = match pipeline.get_field(ObjString(ATTRIBUTES.into())) {
                    Some(Dictionary(attrs)) => attrs,
                    _ => HashMap::new(),
                };

                // Uniform object
                let uniform = pipeline
                    .get_field(ObjString(UNIFORM.into()))
                    .unwrap_or(Dictionary(HashMap::new()));

                // Vertex layout and keys
                let layout_list = match vertex.get_field(ObjString(LAYOUT.into())) {
                    Some(List(layout)) => layout,
                    _ => continue,
                };

                let mut layout = Vec::with_capacity(layout_list.len());
                let mut keys = Vec::with_capacity(layout_list.len() * 2);

                for obj in layout_list {
                    if let ObjString(s) = obj {
                        layout.push(s.clone());
                        match s.as_str() {
                            "vec2" => keys.extend(["x", "y"]),
                            "uv" => keys.extend(["u", "v"]),
                            _ => {}
                        }
                    }
                }

                // Vertex data
                let data_list = match vertex.get_field(ObjString(DATA.into())) {
                    Some(List(data)) => data,
                    _ => continue,
                };

                let mut data = Vec::with_capacity(data_list.len() * keys.len());

                for obj in data_list {
                    if let Dictionary(fields) = obj {
                        for &key in &keys {
                            match fields.get(key) {
                                Some(Number(n)) => data.push(*n as f32),
                                _ => panic!("Expected number for key {}", key),
                            }
                        }
                    }
                }

                // Convert attributes to String/String only if needed
                let attributes = attributes
                    .iter()
                    .map(|(k, v)| (k.clone(), v.to_string()))
                    .collect::<HashMap<String, String>>();

                // Precompiled uniform + shader + vertex buffer
                let uniforms = self
                    .uniform_generator
                    .write()
                    .unwrap()
                    .generate_uniforms(&uniform, &self.display)
                    .expect("Failed to generate uniforms");

                let render_statement = RenderStatement::new(
                    &self.display,
                    PipelineData {
                        attributes,
                        uniforms,
                    },
                    BuffersData { data, layout },
                )
                .expect("Failed to create render statement");

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
                    target.clear_color(0.0, 0.0, 0.0, 1.0);
                    target
                        .draw(
                            &render_statement.vertex_buffer,
                            &NoIndices(PrimitiveType::TriangleStrip),
                            &program,
                            &render_statement.uniforms,
                            &Default::default(),
                        )
                        .unwrap();
                    target.finish().unwrap();
                }
            }
            _ => {}
        }
    }
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if self.render_statement.is_some() {
            self.window.request_redraw();
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    }
}
