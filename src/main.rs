use crate::cli::Cli;
use clap::Parser;
use yun_gl_lib::interpreter::error::Result;
use yun_gl_lib::interpreter::Interpreter;

mod cli;
#[cfg(test)]
mod test;
fn main() -> Result<()> {
    let cli = Cli::parse();
    Interpreter::default().run_script(cli.get_path())
}