#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate derivative;
#[macro_use]
extern crate tracing;

use std::path::Path;

use parser::*;

pub use crate::compiler::Value;
use crate::compiler::{Error, Source};

mod compiler;
mod parser;

pub fn parse_string(input: &str) -> Result<Value, Error> {
    parse_source(Source::from_string(input.to_string()))
}

pub fn parse_file(file_name: &str) -> Result<Value, Error> {
    parse_source(Source::from_file(Path::new(file_name))?)
}

fn parse_source(source: Source) -> Result<Value, Error> {
    let input = source.as_str();
    let (rest, expr) = parse_unit(input).map_err(|e| anyhow!("Cannot parse {}", e))?;
    if !rest.is_empty() {
        bail!("Cannot parse: '{}'", rest);
    }
    compiler::compile(&expr, source.clone())
}
