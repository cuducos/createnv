use std::env::args;

use anyhow::Result;
use createnv::{model_to_text, tokenize};

fn main() -> Result<()> {
    if let Some(path) = args().nth(1) {
        tokenize(&path)?;

        return Ok(());
    }

    model_to_text()
}
