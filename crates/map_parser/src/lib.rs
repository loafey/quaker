#![feature(let_chains)]

mod tokenizer;
use std::io::Result;

pub use tokenizer::tokenizer;
mod parser;
pub use parser::{parser, Entity};

pub fn parse(str: &str) -> Result<Vec<Entity>> {
    parser(tokenizer(str))
}
