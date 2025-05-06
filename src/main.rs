use crate::cli::Cli;
use clap::Parser;
use glium::backend::glutin::SimpleWindowBuilder;
use glium::winit::event_loop::EventLoopBuilder;
use std::sync::Arc;
use yun_gl_lib::app::App;
use yun_gl_lib::interpreter::error::Result;
use yun_gl_lib::interpreter::event::InterpreterEvent;
use yun_gl_lib::rc;

mod cli;

fn main() -> Result<()> {
    let cli = Cli::parse();

    let event_loop = EventLoopBuilder::<InterpreterEvent>::default().build()?;

    let (window, display) = SimpleWindowBuilder::new()
        .with_title("App")
        .build(&event_loop);

    let mut app = App::new(
        rc!(window),
        rc!(display),
        event_loop.create_proxy(),
        cli.get_path().clone(),
    );
    Ok(event_loop.run_app(&mut app)?)
}
