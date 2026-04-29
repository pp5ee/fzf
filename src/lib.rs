pub mod algo;
pub mod ansi;
pub mod cache;
pub mod core;
pub mod functions;
pub mod history;
pub mod item;
pub mod matcher;
pub mod merger;
pub mod options;
pub mod pattern;
pub mod reader;
pub mod result;
pub mod server;
pub mod terminal;
pub mod tokenizer;
pub mod tui;
pub mod util;

pub use options::Options;

use anyhow::Result;

pub fn run(options: Options) -> Result<i32> {
    core::run(options)
}

pub fn parse_options(args: &[String]) -> Result<Options> {
    options::parse_options(args)
}
