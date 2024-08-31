use std::{
    env::args,
    fs::File,
    io::{stdin, Write},
};

use anyhow::Result;
use parser::Parser;

mod model;
mod parser;

const DEFAULT_ENV_SAMPLE: &str = ".env.sample";
const DEFAULT_ENV: &str = ".env";

fn main() -> Result<()> {
    let path = args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_ENV_SAMPLE.to_string());
    let mut parser = Parser::new(path.as_str())?;
    parser.parse(&mut stdin().lock())?;
    let mut output = File::create(DEFAULT_ENV)?;
    output.write_all(parser.to_string().as_bytes())?;
    Ok(())
}
