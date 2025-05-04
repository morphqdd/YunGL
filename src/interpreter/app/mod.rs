use glium::winit::application::ApplicationHandler;
use glium::winit::event::WindowEvent;
use glium::winit::event_loop::ActiveEventLoop;
use glium::winit::window::WindowId;

pub struct Application {}

impl Application {
    pub fn new() -> Self {
        Self {}
    }
}

impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {

    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            _ => {}
        }
    }
}