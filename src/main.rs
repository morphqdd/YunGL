use crate::cli::Cli;
use clap::Parser;
use glium::backend::glutin::SimpleWindowBuilder;
use glium::winit::event_loop::{EventLoop, EventLoopBuilder};
use yun_gl_lib::interpreter::Interpreter;
use yun_gl_lib::interpreter::error::Result;
use yun_gl_lib::interpreter::event::InterpreterEvent;

mod cli;
#[cfg(test)]
mod test;
fn main() -> Result<()> {
    let cli = Cli::parse();

    let event_loop = EventLoopBuilder::<InterpreterEvent>::default().build()?;

    let (window, display) = SimpleWindowBuilder::new()
        .with_title("App")
        .build(&event_loop);

    let mut interpreter = Interpreter::new(
        window,
        display,
        event_loop.create_proxy(),
        cli.get_path().clone(),
    );
    Ok(event_loop.run_app(&mut interpreter)?)
}
