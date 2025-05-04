use crate::cli::Cli;
use clap::Parser;
use yun_gl_lib::interpreter::error::Result;
use yun_gl_lib::interpreter::Interpreter;

mod cli;
#[cfg(test)]
mod test;
fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.get_path() {
        None => Interpreter::default().run_shell(),
        Some(path_to_script) => Interpreter::default().run_script(path_to_script),
    }
}