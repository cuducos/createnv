mod model;
mod parser;

use anyhow::Result;
use model::model_to_text_cli;
use parser::parser_cli;

pub const DEFAULT_ENV_SAMPLE: &str = ".env.sample";
pub const DEFAULT_ENV: &str = ".env";

pub fn model_to_text() -> Result<()> {
    model_to_text_cli()
}

pub fn parser(path: &str) -> Result<()> {
    parser_cli(path)
}
