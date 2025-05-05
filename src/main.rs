use crate::cli::Cli;
use clap::Parser;
use glium::backend::glutin::SimpleWindowBuilder;
use glium::winit::event_loop::EventLoop;
use yun_gl_lib::interpreter::error::Result;
use yun_gl_lib::interpreter::Interpreter;

mod cli;
#[cfg(test)]
mod test;
fn main() -> Result<()> {
    let cli = Cli::parse();

    let event_loop = EventLoop::new()?;

    let (window, display) = SimpleWindowBuilder::new()
        .with_title("App")
        .build(&event_loop);

    let mut interpreter = Interpreter::new(window, display, cli.get_path().clone());
    Ok(event_loop.run_app(&mut interpreter)?)
}