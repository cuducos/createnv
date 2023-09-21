mod model;
mod reader;
mod tokenizer;

use anyhow::Result;

use crate::model::model_to_text_cli;
use crate::tokenizer::tokenize_cli;

pub fn tokenize(path: &String) -> Result<()> {
    tokenize_cli(path)
}

pub fn model_to_text() -> Result<()> {
    model_to_text_cli()
}
