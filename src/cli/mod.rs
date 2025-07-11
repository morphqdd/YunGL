use clap::Parser;
use std::path::PathBuf;

#[derive(Clone, Debug, Parser)]
pub struct Cli {
    path: PathBuf,
}

impl Cli {
    pub fn get_path(&self) -> &PathBuf {
        &self.path
    }
}
