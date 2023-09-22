mod model;
mod parser;
mod reader;
mod tokenizer;

use anyhow::Result;
use model::model_to_text_cli;
use parser::parser_cli;
use tokenizer::tokenize_cli;

pub fn tokenize(path: &String) -> Result<()> {
    tokenize_cli(path)
}

pub fn model_to_text() -> Result<()> {
    model_to_text_cli()
}
pub fn parser(path: &String) -> Result<()> {
    parser_cli(path)
}
