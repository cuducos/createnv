use std::env::args;

use anyhow::Result;
use createnv::{model_to_text, parser, DEFAULT_ENV_SAMPLE};

fn main() -> Result<()> {
    let path = args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_ENV_SAMPLE.to_string());
    if path == "--debug" {
        model_to_text()?;
        return Ok(());
    }
    parser(&path)?;
    Ok(())
}
